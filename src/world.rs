use crate::{
    block::Block,
    camera_distance::CameraDistance,
    chunk::{
        data::ChunkData, index::ChunkIndex, position::ChunkPosition, stage::Stage, Chunk,
        CHUNK_SIZE,
    },
};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use noise::NoiseFn;
use seed::{LoadSeed, WorldSeed};
use std::collections::HashSet;
use world_noise::WorldGenNoise;

const CHUNK_LOAD_DISTANCE_HORIZONTAL: i32 = 2;
const CHUNK_LOAD_DISTANCE_VERTICAL: i32 = 2;

mod seed;
mod world_noise;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(seed::SeedPlugin)
            .init_resource::<ChunkLoadTasks>()
            .add_systems(Startup, init_noise.after(LoadSeed))
            .add_systems(
                Update,
                (
                    begin_chunk_load_tasks,
                    receive_chunk_load_tasks,
                    (update_chunks, despawn_chunks).chain(),
                    generate_terrain,
                )
                    .chain()
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

struct ChunkLoadTaskData {
    entity: Entity,
    chunk: Chunk,
}

#[derive(Resource, Default)]
struct ChunkLoadTasks(HashMap<ChunkPosition, Task<ChunkLoadTaskData>>);

#[derive(Component)]
struct NeedsChunkData;

fn begin_chunk_load_tasks(
    mut tasks: ResMut<ChunkLoadTasks>,
    q_chunk: Query<(Entity, &ChunkPosition), (Without<Chunk>, With<NeedsChunkData>)>,
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
            let chunk = generate_chunk_noise(cloned_noise, pos_ivec);
            ChunkLoadTaskData { entity, chunk }
        });
        tasks.0.insert(*pos, task);
    }
}

fn receive_chunk_load_tasks(mut commands: Commands, mut tasks: ResMut<ChunkLoadTasks>) {
    tasks.0.retain(|_, task| {
        let Some(data) = block_on(future::poll_once(task)) else {
            return true;
        };
        if let Some(mut entity) = commands.get_entity(data.entity) {
            entity
                .insert(data.chunk)
                .remove::<NeedsChunkData>();
        };
        return false;
    });
}

fn kill_tasks_for_unloaded_chunks(
    trigger: Trigger<OnRemove, Chunk>,
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
    q_camera_position: Query<&GlobalTransform, (With<Camera3d>, Changed<ChunkPosition>)>,
    q_chunk_position: Query<(Entity, &ChunkPosition), Or<(With<Chunk>, With<NeedsChunkData>)>>,
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
            NeedsChunkData,
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
        .for_each(|(entity, _, _)| commands.entity(entity).despawn());
}

fn generate_chunk_noise(noise: WorldGenNoise, chunk_pos: IVec3) -> Chunk {
    const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
    let chunk_data = ChunkData {
        stage: Stage::Noise,
        blocks: std::iter::repeat_n(Block::default(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
            .collect::<_>(),
        noise_2d: (0..CHUNK_SIZE * CHUNK_SIZE)
            .map(|idx| {
                let x = idx / CHUNK_SIZE;
                let z = idx % CHUNK_SIZE;
                return noise.get([
                    x as i32 + chunk_pos.x * CHUNK_SIZE_I32,
                    z as i32 + chunk_pos.z * CHUNK_SIZE_I32,
                ]) as f32;
            })
            .collect::<_>(),
    };
    Chunk::new(chunk_data)
}

fn generate_terrain(q_chunk: Query<(&Chunk, &ChunkPosition)>) {
    for (chunk, pos) in q_chunk.iter() {
        generate_terrain_for_chunk(chunk, pos);
    }
}

fn generate_terrain_for_chunk(chunk: &Chunk, pos: &ChunkPosition) {
    const SCALE: f32 = 60.0;
    const DIRT_DEPTH: usize = 2;
    let stage = { chunk.data.read().unwrap().stage };
    if stage != Stage::Noise {
        return;
    }
    let noise = {
        chunk
            .data
            .read()
            .unwrap()
            .noise_2d
            .clone()
    };
    let Ok(mut chunk_data) = chunk.data.write() else {
        return;
    };
    let chunk_pos = pos.0;
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let index = CHUNK_SIZE * x + z;
            let height = (noise[index] * SCALE) as i32 + 1;
            let Some(chunk_height) = Some(height - (chunk_pos.y * CHUNK_SIZE as i32))
                .filter(|h| h >= &0)
                .map(|h| h as usize)
            else {
                continue;
            };
            if chunk_height >= DIRT_DEPTH - 1 {
                for y in (0..=chunk_height - (DIRT_DEPTH - 1)).filter(|h| h < &CHUNK_SIZE) {
                    chunk_data.put(x, y, z, Block::Stone);
                }
            }
            if chunk_height >= DIRT_DEPTH {
                for y in (chunk_height - DIRT_DEPTH..chunk_height).filter(|h| h < &CHUNK_SIZE) {
                    chunk_data.put(x, y, z, Block::Dirt);
                }
            }
            if chunk_height < CHUNK_SIZE {
                chunk_data.put(x, chunk_height, z, Block::Grass);
            }
        }
    }
    chunk_data.stage = Stage::Terrain;
}
