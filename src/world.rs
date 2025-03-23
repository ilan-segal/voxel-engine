use crate::{
    block::Block,
    camera_distance::CameraDistance,
    chunk::{
        data::{Blocks, ContinentNoise, HeightNoise, Noise3d},
        position::ChunkPosition,
        spatial::SpatiallyMapped,
        Chunk, CHUNK_SIZE, CHUNK_SIZE_I32,
    },
    player::Player,
    render_layer::WORLD_LAYER,
    state::GameState,
};
use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::view::RenderLayers,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use index::ChunkIndex;
use seed::{LoadSeed, WorldSeed};
use stage::Stage;
use std::collections::HashSet;
use world_noise::{ContinentNoiseGenerator, HeightNoiseGenerator};

const CHUNK_LOAD_DISTANCE_HORIZONTAL: i32 = 7;
const CHUNK_LOAD_DISTANCE_VERTICAL: i32 = 3;

pub mod block_update;
pub mod index;
pub mod neighborhood;
pub mod seed;
pub mod stage;
mod world_noise;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            seed::SeedPlugin,
            index::ChunkIndexPlugin,
            block_update::BlockPlugin,
            neighborhood::NeighborhoodPlugin::<Blocks>::new(),
            neighborhood::NeighborhoodPlugin::<Stage>::new(),
            neighborhood::NeighborhoodPlugin::<Noise3d>::new(),
        ))
        .init_resource::<ChunkLoadTasks>()
        .add_systems(Startup, init_noise.after(LoadSeed))
        .add_systems(
            Update,
            (
                (update_chunks, despawn_chunks, begin_noise_load_tasks).chain(),
                begin_terrain_sculpt_tasks,
                // begin_structure_load_tasks,
                receive_chunk_load_tasks,
            )
                .in_set(WorldSet)
                .run_if(in_state(GameState::InGame)),
        )
        .observe(kill_tasks_for_unloaded_chunks);
    }
}

fn init_noise(mut commands: Commands, seed: Res<WorldSeed>) {
    commands.insert_resource(ContinentNoiseGenerator::new(seed.0));
    commands.insert_resource(HeightNoiseGenerator::new(seed.0));
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSet;

struct ChunkLoadTaskData {
    entity: Entity,
    added_data: AddedChunkData,
}

enum AddedChunkData {
    Noise {
        continent_noise: ContinentNoise,
        height_noise: HeightNoise,
    },
    Blocks(Blocks, Stage),
}

#[derive(Resource, Default)]
struct ChunkLoadTasks(HashMap<ChunkPosition, Task<ChunkLoadTaskData>>);

fn kill_tasks_for_unloaded_chunks(
    trigger: Trigger<OnRemove, Blocks>,
    index: Res<ChunkIndex>,
    mut tasks: ResMut<ChunkLoadTasks>,
) {
    if let Some(pos) = index
        .pos_by_entity
        .get(&trigger.entity())
    {
        tasks.0.remove(&ChunkPosition(*pos));
    }
}

fn update_chunks(
    mut commands: Commands,
    q_camera_position: Query<&GlobalTransform, (With<Player>, Changed<ChunkPosition>)>,
    q_chunk_position: Query<(Entity, &ChunkPosition), With<Chunk>>,
) {
    let Ok(pos) = q_camera_position.get_single() else {
        return;
    };
    let camera_position = pos.compute_transform().translation;
    let chunk_pos = ChunkPosition::from_world_position(&camera_position);
    // Determine position of chunks that should be loaded
    let mut should_be_loaded_positions: HashSet<IVec3> = HashSet::new();
    for chunk_x in -CHUNK_LOAD_DISTANCE_HORIZONTAL..=CHUNK_LOAD_DISTANCE_HORIZONTAL {
        for chunk_z in -CHUNK_LOAD_DISTANCE_HORIZONTAL..=CHUNK_LOAD_DISTANCE_HORIZONTAL {
            for chunk_y in -CHUNK_LOAD_DISTANCE_VERTICAL..=CHUNK_LOAD_DISTANCE_VERTICAL {
                let cur_chunk_pos =
                    ChunkPosition(chunk_pos.0 + IVec3::new(chunk_x, chunk_y, chunk_z));
                should_be_loaded_positions.insert(cur_chunk_pos.0);
            }
        }
    }
    // Iterate over loaded chunks, despawning any which shouldn't be loaded right now
    // By removing loaded chunks from our hash set, we are left only with the chunks
    // which need to be loaded.
    for (entity, chunk_pos) in q_chunk_position.iter() {
        if !should_be_loaded_positions.remove(&chunk_pos.0) {
            // The chunk should be unloaded since it's not in our set
            commands
                .entity(entity)
                .insert(ToDespawn);
        }
    }
    // Finally, load the new chunks
    for pos in should_be_loaded_positions {
        commands.spawn((
            Chunk,
            ChunkPosition(pos),
            SpatialBundle {
                transform: Transform {
                    translation: (pos * CHUNK_SIZE as i32).as_vec3() + Vec3::Y,
                    scale: Vec3::ONE * super::BLOCK_SIZE,
                    ..Default::default()
                },
                visibility: Visibility::Visible,
                ..default()
            },
            RenderLayers::layer(WORLD_LAYER),
        ));
    }
}

#[derive(Component)]
struct ToDespawn;

const CHUNKS_DESPAWNED_PER_FRAME: usize = 10;

fn despawn_chunks(q_chunk: Query<(Entity, &ToDespawn, &CameraDistance)>, mut commands: Commands) {
    q_chunk
        .iter()
        // Descending order (highest distance first)
        .sort_by::<&CameraDistance>(|a, b| b.0.partial_cmp(&a.0).unwrap())
        .take(CHUNKS_DESPAWNED_PER_FRAME)
        .for_each(|(entity, _, _)| commands.entity(entity).despawn_recursive());
}

fn receive_chunk_load_tasks(mut commands: Commands, mut tasks: ResMut<ChunkLoadTasks>) {
    tasks.0.retain(|_, task| {
        let Some(data) = block_on(future::poll_once(task)) else {
            return true;
        };
        let Some(mut entity) = commands.get_entity(data.entity) else {
            return false;
        };
        match data.added_data {
            AddedChunkData::Noise {
                continent_noise,
                height_noise,
            } => {
                entity.try_insert((continent_noise, height_noise, Stage::Noise));
            }
            AddedChunkData::Blocks(blocks, stage) => {
                entity.try_insert((blocks, stage));
            }
        }
        return false;
    });
}

fn begin_noise_load_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<(Entity, &ChunkPosition), (With<Chunk>, Without<ContinentNoise>)>,
    continent_noise_generator: Res<ContinentNoiseGenerator>,
    height_noise_generator: Res<HeightNoiseGenerator>,
) {
    for (entity, pos) in q_chunk.iter() {
        if tasks.0.contains_key(pos) {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let continent_noise_generator = continent_noise_generator.clone();
        let height_noise_generator = height_noise_generator.clone();
        let pos_ivec = pos.0;
        let task = task_pool.spawn(async move {
            let (continent_noise, height_noise) =
                generate_chunk_noise(pos_ivec, continent_noise_generator, height_noise_generator);
            ChunkLoadTaskData {
                entity,
                added_data: AddedChunkData::Noise {
                    continent_noise,
                    height_noise,
                },
            }
        });
        tasks.0.insert(*pos, task);
    }
}

fn generate_chunk_noise(
    chunk_pos: IVec3,
    continent_noise: ContinentNoiseGenerator,
    height_noise: HeightNoiseGenerator,
) -> (ContinentNoise, HeightNoise) {
    let continent = ContinentNoise::from((continent_noise.0.as_ref(), chunk_pos));
    let height = HeightNoise::from((height_noise.0.as_ref(), chunk_pos));
    (continent, height)
}

#[derive(QueryData)]
struct TerrainGenerateData {
    entity: Entity,
    chunk_pos: &'static ChunkPosition,
    stage: &'static Stage,
    continent_noise: &'static ContinentNoise,
    height_noise: &'static HeightNoise,
}

fn begin_terrain_sculpt_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<TerrainGenerateData, (With<Chunk>, Without<Blocks>, Changed<Stage>)>,
) {
    for item in q_chunk.iter() {
        if tasks.0.contains_key(item.chunk_pos) || item.stage != &Stage::Noise {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let continent_noise = item.continent_noise.clone();
        let height_noise = item.height_noise.clone();
        let cloned_pos = item.chunk_pos.clone();
        let task = task_pool.spawn(async move {
            let blocks =
                generate_terrain_sculpt_for_chunk(cloned_pos, continent_noise, height_noise);
            ChunkLoadTaskData {
                entity: item.entity,
                added_data: AddedChunkData::Blocks(blocks, Stage::Sculpt),
            }
        });
        tasks.0.insert(*item.chunk_pos, task);
    }
}

fn generate_terrain_sculpt_for_chunk(
    pos: ChunkPosition,
    continent: ContinentNoise,
    height: HeightNoise,
) -> Blocks {
    const CONTINENT_SCALE: f32 = 60.0;
    const LAND_HEIGHT_SCALE: f32 = 50.0;
    const SEA_LEVEL: i32 = 0;
    let chunk_pos = pos.0;
    Blocks::from_fn(|[x, y, z]| {
        let yi = chunk_pos.y * CHUNK_SIZE_I32 + y as i32;
        let y = yi as f32;
        let continent_noise = (continent.at_pos([x, z]) - 0.5) * 2.0;
        if continent_noise <= 0.0 {
            return if y < continent_noise * CONTINENT_SCALE {
                Block::Stone
            } else if yi <= SEA_LEVEL {
                Block::Water
            } else {
                Block::Air
            };
        }
        let coast_height_factor = stretch_range_onto_unit_interval(continent_noise, 0.0, 0.2);
        let height_noise = height.at_pos([x, z]);
        if y < height_noise * coast_height_factor * LAND_HEIGHT_SCALE {
            Block::Stone
        } else {
            Block::Air
        }
    })
}

fn stretch_range_onto_unit_interval(value: f32, a: f32, b: f32) -> f32 {
    let range_size = b - a;
    let scaled_value = (value - a) / range_size;
    return clamp(scaled_value, 0., 1.);
}

fn clamp(value: f32, a: f32, b: f32) -> f32 {
    value.max(a).min(b)
}

// fn begin_structure_load_tasks(
//     mut tasks: ResMut<ChunkLoadTasks>,
//     index: Res<ChunkIndex>,
//     q_chunk: Query<(Entity, &ChunkPosition, &Stage), With<Chunk>>,
// ) {
//     for (entity, pos, stage) in q_chunk.iter() {
//         if tasks.0.contains_key(pos) || stage != &Stage::Terrain {
//             continue;
//         }
//         let neighborhood = index.get_neighborhood(&pos.0);
//         if neighborhood.get_lowest_stage() < Stage::Terrain
//             || neighborhood
//                 .iter_chunks()
//                 .any(|c| c.is_none())
//         {
//             continue;
//         }
//         let task_pool = AsyncComputeTaskPool::get();
//         let task = task_pool.spawn(async move {
//             let blocks = generate_structures(neighborhood);
//             ChunkLoadTaskData {
//                 entity,
//                 added_data: AddedChunkData::Blocks(blocks, Stage::Structures),
//             }
//         });
//         tasks.0.insert(*pos, task);
//     }
// }

// fn generate_structures(blocks: &Neighborhood<Blocks>, noise: &Neighborhood<Noise3d>) -> Blocks {
//     let mut middle_blocks_chunk = blocks
//         .middle_chunk()
//         .clone()
//         .expect("Middle chunk")
//         .as_ref()
//         .clone();
//     let structure_types = vec![StructureType::Tree];
//     let structure_blocks = structure_types
//         .iter()
//         .flat_map(|s| s.get_structures(blocks, noise))
//         .flat_map(|(structure, [x0, y0, z0])| {
//             structure
//                 .get_blocks()
//                 .iter()
//                 .map(move |(block, [x1, y1, z1])| (*block, [x0 + x1, y0 + y1, z0 + z1]))
//                 .collect::<Vec<_>>()
//         })
//         .filter(|(_, [x, y, z])| {
//             let x = *x;
//             let y = *y;
//             let z = *z;
//             0 <= x
//                 && x < CHUNK_SIZE as i32
//                 && 0 <= y
//                 && y < CHUNK_SIZE as i32
//                 && 0 <= z
//                 && z < CHUNK_SIZE as i32
//         })
//         .collect::<Vec<_>>();
//     // TODO: Vary available structure types by looking at local data
//     for (block, [x, y, z]) in structure_blocks.iter().copied() {
//         let x = x as usize;
//         let y = y as usize;
//         let z = z as usize;
//         *middle_blocks_chunk.at_pos_mut([x, y, z]) = block;
//     }
//     return middle_blocks_chunk;
// }
