use crate::{
    block::Block,
    camera_distance::CameraDistance,
    chunk::{
        data::{
            Blocks, ContinentNoise, FromNoise, HeightNoise, HumidityNoise, Noise3d,
            TemperatureNoise, Terrain,
        },
        position::ChunkPosition,
        spatial::SpatiallyMapped,
        Chunk, CHUNK_LENGTH, CHUNK_SIZE_I32,
    },
    player::Player,
    render_layer::WORLD_LAYER,
    state::GameState,
    structure::StructureType,
    world::neighborhood::CompleteNeighborhood,
};
use bevy::{
    ecs::query::QueryData,
    platform::collections::HashMap,
    prelude::*,
    render::view::RenderLayers,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use index::ChunkIndex;
use neighborhood::Neighborhood;
use noise::NoiseFn;
use seed::{LoadSeed, WorldSeed};
use stage::Stage;
use std::collections::HashSet;
use world_noise::{
    CaveNetworkNoiseGenerator, ClimateNoise, ContinentNoiseGenerator, HeightNoiseGenerator,
    WhiteNoise,
};

const CHUNK_LOAD_DISTANCE_HORIZONTAL: i32 = 5;
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
            neighborhood::NeighborhoodPlugin::<Terrain>::new(),
            neighborhood::NeighborhoodPlugin::<Blocks>::new(),
            neighborhood::NeighborhoodPlugin::<Stage>::new(),
            neighborhood::NeighborhoodPlugin::<Noise3d>::new(),
        ))
        .init_resource::<ChunkLoadTasks>()
        .add_systems(Startup, init_noise.after(LoadSeed))
        .add_systems(
            Update,
            (
                (update_chunks, despawn_chunks).chain(),
                receive_chunk_load_tasks,
                begin_noise_load_tasks,
                begin_terrain_sculpt_tasks,
                begin_structure_load_tasks,
            )
                .in_set(WorldSet)
                .run_if(in_state(GameState::InGame)),
        )
        .add_observer(kill_tasks_for_unloaded_chunks);
    }
}

fn init_noise(mut commands: Commands, seed: Res<WorldSeed>) {
    commands.insert_resource(ContinentNoiseGenerator::new(seed.0));
    commands.insert_resource(HeightNoiseGenerator::new(seed.0));
    commands.insert_resource(WhiteNoise::new(seed.0));
    commands.insert_resource(CaveNetworkNoiseGenerator::new(seed.0));
    commands.insert_resource(ClimateNoise::new(seed.0, 1000.0))
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSet;

struct ChunkLoadTaskData {
    entity: Entity,
    added_data: AddedChunkData,
}

enum AddedChunkData {
    Noise(NoiseBundle),
    Terrain(Terrain),
    // Blocks(Blocks, Stage),
    BlockUpdates(Vec<(Block, [usize; 3])>, Stage),
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
        .get(&trigger.target())
    {
        tasks.0.remove(&ChunkPosition(*pos));
    }
}

fn update_chunks(
    mut commands: Commands,
    q_camera_position: Query<&GlobalTransform, (With<Player>, Changed<ChunkPosition>)>,
    q_chunk_position: Query<(Entity, &ChunkPosition), With<Chunk>>,
) {
    let Ok(pos) = q_camera_position.single() else {
        return;
    };
    let camera_position = pos.compute_transform().translation;
    // let camera_position = Vec3::ZERO;
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
            Transform::from_translation((pos * CHUNK_SIZE_I32).as_vec3() + Vec3::Y),
            Visibility::Visible,
            RenderLayers::layer(WORLD_LAYER),
        ));
    }
}

#[derive(Component)]
struct ToDespawn;

const CHUNKS_DESPAWNED_PER_FRAME: usize = 10;

fn despawn_chunks(
    q_chunk: Query<(Entity, &CameraDistance), (With<ToDespawn>, With<Chunk>)>,
    mut commands: Commands,
) {
    q_chunk
        .iter()
        // Descending order (highest distance first)
        .sort::<&CameraDistance>()
        .take(CHUNKS_DESPAWNED_PER_FRAME)
        .for_each(|(entity, _)| commands.entity(entity).despawn());
}

fn receive_chunk_load_tasks(
    mut commands: Commands,
    mut tasks: ResMut<ChunkLoadTasks>,
    mut q_blocks: Query<&mut Blocks>,
) {
    tasks.0.retain(|_, task| {
        let Some(data) = block_on(future::poll_once(task)) else {
            return true;
        };
        let Ok(mut entity) = commands.get_entity(data.entity) else {
            return false;
        };
        match data.added_data {
            AddedChunkData::Noise(bundle) => {
                entity.try_insert((Stage::Noise, bundle));
            }
            AddedChunkData::Terrain(terrain) => {
                entity.try_insert((terrain.clone(), Stage::Sculpt, Blocks(terrain.0)));
            }
            // AddedChunkData::Blocks(blocks, stage) => {
            //     entity.try_insert((blocks, stage));
            // }
            AddedChunkData::BlockUpdates(block_updates, stage) => {
                let Ok(blocks) = &mut q_blocks.get_mut(data.entity) else {
                    log::warn!("Failed to get Blocks component during worldgen update");
                    return false;
                };
                blocks.set_changed();
                block_updates
                    .iter()
                    .for_each(|(block, pos)| {
                        *blocks.at_pos_mut(*pos) = *block;
                    });
                entity.try_insert(stage);
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
    white_noise: Res<WhiteNoise>,
    climate_noise: Res<ClimateNoise>,
) {
    for (entity, pos) in q_chunk.iter() {
        if tasks.0.contains_key(pos) {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let continent_noise_generator = continent_noise_generator.clone();
        let height_noise_generator = height_noise_generator.clone();
        let white_noise = white_noise.clone();
        let climate_noise = climate_noise.clone();
        // let cave_noise_generator = cave_noise_generator.clone();
        let pos_ivec = pos.0;
        let task = task_pool.spawn(async move {
            let bundle = generate_chunk_noise(
                pos_ivec,
                continent_noise_generator,
                height_noise_generator,
                white_noise,
                climate_noise,
            );
            ChunkLoadTaskData {
                entity,
                added_data: AddedChunkData::Noise(bundle),
            }
        });
        tasks.0.insert(*pos, task);
    }
}

#[derive(Bundle)]
struct NoiseBundle {
    continent: ContinentNoise,
    height: HeightNoise,
    white: Noise3d,
    temperature: TemperatureNoise,
    humidity: HumidityNoise,
    // cave: CaveNetworkNoise,
}

fn generate_chunk_noise(
    chunk_pos: IVec3,
    continent_noise: ContinentNoiseGenerator,
    height_noise: HeightNoiseGenerator,
    white_noise: WhiteNoise,
    climate_noise: ClimateNoise, // cave_noise_generator: CaveNetworkNoiseGenerator,
) -> NoiseBundle {
    let continent = ContinentNoise::from_noise(continent_noise.0.as_ref(), chunk_pos);
    let height = HeightNoise::from_noise(height_noise.0.as_ref(), chunk_pos);
    let white = Noise3d::from_noise(white_noise, chunk_pos);
    let temperature = TemperatureNoise::from_noise(climate_noise.temperature.as_ref(), chunk_pos);
    let humidity = HumidityNoise::from_noise(climate_noise.humidity.as_ref(), chunk_pos);
    // let cave = CaveNetworkNoise::from((cave_noise_generator, chunk_pos));
    NoiseBundle {
        continent,
        height,
        white,
        temperature,
        humidity,
    }
}

#[derive(QueryData)]
struct TerrainGenerateData {
    entity: Entity,
    chunk_pos: &'static ChunkPosition,
    stage: &'static Stage,
    continent_noise: &'static ContinentNoise,
    height_noise: &'static HeightNoise,
    noise: &'static Noise3d,
    // cave_network_noise: &'static CaveNetworkNoise,
}

fn begin_terrain_sculpt_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<TerrainGenerateData, (With<Chunk>, Without<Blocks>, Changed<Stage>)>,
    cave_noise: Res<CaveNetworkNoiseGenerator>,
) {
    for item in q_chunk.iter() {
        if tasks.0.contains_key(item.chunk_pos) || item.stage != &Stage::Noise {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let continent_noise = item.continent_noise.clone();
        let height_noise = item.height_noise.clone();
        // let cave_network_noise = item.cave_network_noise.clone();
        let cloned_pos = item.chunk_pos.clone();
        let cloned_cave_noise = cave_noise.clone();
        let cloned_noise = item.noise.clone();
        let task = task_pool.spawn(async move {
            let blocks = generate_terrain_sculpt_for_chunk(
                cloned_pos,
                continent_noise,
                height_noise,
                cloned_cave_noise,
                cloned_noise,
            );
            ChunkLoadTaskData {
                entity: item.entity,
                added_data: AddedChunkData::Terrain(blocks),
            }
        });
        tasks.0.insert(*item.chunk_pos, task);
    }
}

const BEDROCK_DEPTH_CHUNKS: i32 = -5;
const MAX_DEPTH: i32 = BEDROCK_DEPTH_CHUNKS * CHUNK_SIZE_I32;

fn generate_terrain_sculpt_for_chunk(
    pos: ChunkPosition,
    continent: ContinentNoise,
    height: HeightNoise,
    cave_noise: CaveNetworkNoiseGenerator,
    noise: Noise3d,
    // cave_network_noise: CaveNetworkNoise,
) -> Terrain {
    if pos.0.y < BEDROCK_DEPTH_CHUNKS {
        return Terrain(vec![Block::Air; CHUNK_LENGTH]);
    }

    const CONTINENT_SCALE: f32 = 60.0;
    const LAND_HEIGHT_SCALE: f32 = 50.0;
    const SEA_LEVEL: i32 = 0;
    const DIRT_DEPTH: i32 = 4;
    const SEA_SAND_DEPTH: f32 = 2.0;
    let chunk_pos = pos.0;
    Terrain::from_fn(|pos| {
        let [x, _, z] = pos;
        let world_pos = chunk_pos * CHUNK_SIZE_I32 + IVec3::from(pos.map(|x| x as i32));
        let bedrock_offset = if noise.at_pos([x, 0, z]) < &0.5 { 0 } else { 1 };
        if world_pos.y < MAX_DEPTH {
            return Block::Air;
        } else if world_pos.y <= MAX_DEPTH + bedrock_offset {
            return Block::Bedrock;
        }
        let y = world_pos.y as f32;
        let continent_noise = (continent.at_pos([x, z]) - 0.5) * 2.0;
        let cave_noise = cave_noise.get(world_pos.into());
        let cave_threshold = get_cave_threshold(world_pos.y);
        let is_cave = cave_noise < cave_threshold;
        // Ocean
        if continent_noise <= 0.0 {
            return if y < continent_noise * CONTINENT_SCALE {
                if is_cave {
                    Block::Air
                } else if y < continent_noise * CONTINENT_SCALE - SEA_SAND_DEPTH {
                    Block::Stone
                } else {
                    Block::Sand
                }
            } else if world_pos.y <= SEA_LEVEL {
                Block::Water
            } else {
                Block::Air
            };
        }
        // Land
        let coast_height_factor = stretch_range_onto_unit_interval(continent_noise, 0.0, 0.2);
        let height_noise = height.at_pos([x, z]);
        let land_height = height_noise * coast_height_factor * LAND_HEIGHT_SCALE;
        let is_coast = land_height <= 2.0;
        if y < land_height && is_cave {
            return Block::Air;
        }
        if y < land_height - DIRT_DEPTH as f32 {
            Block::Stone
        } else if y < land_height - 1.0 {
            Block::Dirt
        } else if y < land_height {
            if is_coast {
                Block::Sand
            } else {
                Block::Grass
            }
        } else {
            Block::Air
        }
    })
}

fn get_cave_threshold(height: i32) -> f64 {
    const MIN_DEPTH_THRESHOLD: f64 = 0.095;
    const MAX_DEPTH_THRESHOLD: f64 = 0.100;
    const DEPTH_START: i32 = -CHUNK_SIZE_I32;
    const DEPTH_END: i32 = -CHUNK_SIZE_I32 * 8;
    const BUFFER_BEFORE_BEDROCK: i32 = 4;
    const CAVE_TAPER_THICKNESS: i32 = 4;
    const CAVE_TAPER_END: i32 = MAX_DEPTH + BUFFER_BEFORE_BEDROCK;
    const CAVE_TAPER_START: i32 = CAVE_TAPER_END + CAVE_TAPER_THICKNESS;

    let buffer_t = stretch_range_onto_unit_interval(
        height as f32,
        CAVE_TAPER_END as f32,
        CAVE_TAPER_START as f32,
    );

    let t = stretch_range_onto_unit_interval(height as f32, DEPTH_END as f32, DEPTH_START as f32)
        as f64;
    return MAX_DEPTH_THRESHOLD.lerp(MIN_DEPTH_THRESHOLD, t) * buffer_t as f64;
}

fn stretch_range_onto_unit_interval(value: f32, a: f32, b: f32) -> f32 {
    let range_size = b - a;
    let scaled_value = (value - a) / range_size;
    return scaled_value.clamp(0., 1.);
}

#[derive(QueryData)]
struct StructureQueryData {
    entity: Entity,
    pos: &'static ChunkPosition,
    stage: &'static Stage,
    terrain_neighborhood: &'static Neighborhood<Terrain>,
    noise_neighborhood: &'static Neighborhood<Noise3d>,
}

fn begin_structure_load_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<StructureQueryData, (With<Chunk>, With<CompleteNeighborhood<Terrain>>)>,
) {
    for item in q_chunk.iter() {
        if tasks.0.contains_key(item.pos) || item.stage != &Stage::Sculpt {
            continue;
        }
        let task_pool = AsyncComputeTaskPool::get();
        let blocks = item.terrain_neighborhood.clone();
        let noise = item.noise_neighborhood.clone();
        let entity = item.entity;
        let task = task_pool.spawn(async move {
            let added_data = generate_structures(blocks, noise);
            ChunkLoadTaskData { entity, added_data }
        });
        tasks.0.insert(*item.pos, task);
    }
}

fn generate_structures(
    blocks: Neighborhood<Terrain>,
    noise: Neighborhood<Noise3d>,
) -> AddedChunkData {
    // TODO: Vary structure type by biome
    // let structure = StructureType::Tree;
    // let updates = structure.get_structures(&blocks, &noise);

    AddedChunkData::BlockUpdates(
        StructureType::Tree.get_structure_blocks(&blocks, &noise),
        Stage::Structures,
    )
}
