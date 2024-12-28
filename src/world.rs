use crate::{
    block::Block,
    camera_distance::CameraDistance,
    chunk::{
        data::{Blocks, Noise3d, Perlin2d},
        position::ChunkPosition,
        spatial::SpatiallyMapped,
        Chunk, CHUNK_SIZE, CHUNK_SIZE_I32,
    },
    player::Player,
    structure::StructureType,
};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use index::ChunkIndex;
use neighborhood::ChunkNeighborhood;
use noise::NoiseFn;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use seed::{LoadSeed, WorldSeed};
use stage::Stage;
use std::collections::HashSet;
use world_noise::WorldGenNoise;

const CHUNK_LOAD_DISTANCE_HORIZONTAL: i32 = 10;
const CHUNK_LOAD_DISTANCE_VERTICAL: i32 = 10;

pub mod block_update;
pub mod index;
pub mod neighborhood;
mod seed;
pub mod stage;
mod world_noise;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            seed::SeedPlugin,
            index::ChunkIndexPlugin,
            block_update::BlockPlugin,
        ))
        .init_resource::<ChunkLoadTasks>()
        .add_systems(Startup, init_noise.after(LoadSeed))
        .add_systems(
            Update,
            (
                (update_chunks, despawn_chunks, begin_noise_load_tasks).chain(),
                begin_terrain_load_tasks,
                begin_structure_load_tasks,
                receive_chunk_load_tasks,
            )
                .in_set(WorldSet),
        )
        .observe(kill_tasks_for_unloaded_chunks);
    }
}

fn init_noise(mut commands: Commands, seed: Res<WorldSeed>) {
    commands.insert_resource(WorldGenNoise::new(seed.0));
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSet;

#[derive(Bundle)]
pub struct ChunkBundle {
    pub stage: Stage,
    pub blocks: Blocks,
    pub perlin_2d: Perlin2d,
    pub noise_3d: Noise3d,
}

struct ChunkLoadTaskData {
    entity: Entity,
    added_data: AddedChunkData,
}

enum AddedChunkData {
    Noise {
        perlin_2d: Perlin2d,
        noise_3d: Noise3d,
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
        .sort_by::<&CameraDistance>(|a, b| {
            b.0
                .partial_cmp(&a.0)
                .unwrap()
        })
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
                perlin_2d,
                noise_3d,
            } => {
                entity.insert((perlin_2d, noise_3d, Stage::Noise));
            }
            AddedChunkData::Blocks(blocks, stage) => {
                entity.insert((blocks, stage));
            }
        }
        return false;
    });
}

fn begin_noise_load_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<(Entity, &ChunkPosition), (With<Chunk>, Without<Noise3d>, Without<Perlin2d>)>,
    world_gen_noise: Res<WorldGenNoise>,
) {
    for (entity, pos) in q_chunk.iter() {
        if tasks.0.contains_key(pos) {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let cloned_noise = world_gen_noise.clone();
        let pos_ivec = pos.0;
        let task = task_pool.spawn(async move {
            let (perlin_2d, noise_3d) = generate_chunk_noise(cloned_noise, pos_ivec);
            ChunkLoadTaskData {
                entity,
                added_data: AddedChunkData::Noise {
                    perlin_2d,
                    noise_3d,
                },
            }
        });
        tasks.0.insert(*pos, task);
    }
}

fn generate_chunk_noise(noise: WorldGenNoise, chunk_pos: IVec3) -> (Perlin2d, Noise3d) {
    let perlin_2d = (0..CHUNK_SIZE * CHUNK_SIZE)
        .into_par_iter()
        .map(|idx| {
            let x = idx / CHUNK_SIZE;
            let z = idx % CHUNK_SIZE;
            return noise.get([
                x as i32 + chunk_pos.x * CHUNK_SIZE_I32,
                z as i32 + chunk_pos.z * CHUNK_SIZE_I32,
            ]) as f32;
        })
        .collect::<_>();
    let noise_3d = (0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
        .into_par_iter()
        .map(|idx| {
            let x = idx % (CHUNK_SIZE * CHUNK_SIZE);
            let y = (idx / CHUNK_SIZE) % CHUNK_SIZE;
            let z = idx / (CHUNK_SIZE * CHUNK_SIZE);
            let value = noise.white_noise().get([
                (x as i32 + chunk_pos.x * CHUNK_SIZE_I32),
                (y as i32 + chunk_pos.y * CHUNK_SIZE_I32),
                (z as i32 + chunk_pos.z * CHUNK_SIZE_I32),
            ]) as f32;
            // println!("{}", value);
            return value;
        })
        .collect::<_>();

    (Perlin2d(perlin_2d), Noise3d(noise_3d))
}

fn begin_terrain_load_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<
        (Entity, &ChunkPosition, &Perlin2d, &Stage),
        (With<Chunk>, With<Perlin2d>, Without<Blocks>, Changed<Stage>),
    >,
) {
    for (entity, pos, perlin_2d, stage) in q_chunk.iter() {
        if tasks.0.contains_key(pos) || stage != &Stage::Noise {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let cloned_perlin = perlin_2d.clone();
        let cloned_pos = pos.clone();
        let task = task_pool.spawn(async move {
            let blocks = generate_terrain_for_chunk(cloned_perlin, cloned_pos);
            ChunkLoadTaskData {
                entity,
                added_data: AddedChunkData::Blocks(blocks, Stage::Terrain),
            }
        });
        tasks.0.insert(*pos, task);
    }
}

fn generate_terrain_for_chunk(noise: Perlin2d, pos: ChunkPosition) -> Blocks {
    const SCALE: f32 = 60.0;
    const DIRT_DEPTH: usize = 2;
    let chunk_pos = pos.0;
    let mut blocks = Blocks::default();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = (noise.at_pos([x, z]) * SCALE) as i32 + 1;
            let Some(chunk_height) = Some(height - (chunk_pos.y * CHUNK_SIZE as i32))
                .filter(|h| h >= &0)
                .map(|h| h as usize)
            else {
                continue;
            };
            if chunk_height >= DIRT_DEPTH - 1 {
                for y in (0..=chunk_height - (DIRT_DEPTH - 1)).filter(|h| h < &CHUNK_SIZE) {
                    *blocks.at_pos_mut([x, y, z]) = Block::Stone;
                }
            }
            if chunk_height >= DIRT_DEPTH {
                for y in (chunk_height - DIRT_DEPTH..chunk_height).filter(|h| h < &CHUNK_SIZE) {
                    *blocks.at_pos_mut([x, y, z]) = Block::Dirt;
                }
            }
            if chunk_height < CHUNK_SIZE {
                *blocks.at_pos_mut([x, chunk_height, z]) = Block::Grass;
            }
        }
    }
    return blocks;
}

fn begin_structure_load_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    index: Res<ChunkIndex>,
    q_chunk: Query<(Entity, &ChunkPosition, &Stage), With<Chunk>>,
) {
    for (entity, pos, stage) in q_chunk.iter() {
        if tasks.0.contains_key(pos) || stage != &Stage::Terrain {
            continue;
        }
        let neighborhood = index.get_neighborhood(&pos.0);
        if neighborhood.get_lowest_stage() < Stage::Terrain
            || neighborhood
                .iter_chunks()
                .any(|c| c.is_none())
        {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            let blocks = generate_structures(neighborhood);
            ChunkLoadTaskData {
                entity,
                added_data: AddedChunkData::Blocks(blocks, Stage::Structures),
            }
        });
        tasks.0.insert(*pos, task);
    }
}

fn generate_structures(neighborhood: ChunkNeighborhood) -> Blocks {
    let mut blocks = neighborhood
        .middle()
        .expect("Middle chunk")
        .blocks
        .clone();
    let structure_types = vec![StructureType::Tree];
    let structure_blocks = structure_types
        .iter()
        .flat_map(|s| s.get_structures(&neighborhood))
        .flat_map(|(structure, [x0, y0, z0])| {
            structure
                .get_blocks()
                .iter()
                .map(move |(block, [x1, y1, z1])| (*block, [x0 + x1, y0 + y1, z0 + z1]))
                .collect::<Vec<_>>()
        })
        .filter(|(_, [x, y, z])| {
            let x = *x;
            let y = *y;
            let z = *z;
            0 <= x
                && x < CHUNK_SIZE as i32
                && 0 <= y
                && y < CHUNK_SIZE as i32
                && 0 <= z
                && z < CHUNK_SIZE as i32
        })
        .collect::<Vec<_>>();
    // TODO: Vary available structure types by looking at local data
    for (block, [x, y, z]) in structure_blocks.iter().copied() {
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;
        *blocks.at_pos_mut([x, y, z]) = block;
    }
    return blocks;
}
