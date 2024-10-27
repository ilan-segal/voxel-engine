use bevy::prelude::*;
use noise::NoiseFn;
use std::{collections::HashSet, sync::Arc};

use crate::{
    block::Block,
    chunk::{Chunk, ChunkPosition},
    chunk_data::{ChunkData, CHUNK_SIZE},
    world_noise::WorldGenNoise,
};

const WORLD_SEED: u32 = 0xDEADBEEF;
const CHUNK_LOAD_DISTANCE_HORIZONTAL: i32 = 20;
const CHUNK_LOAD_DISTANCE_VERTICAL: i32 = 2;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGenNoise::new(WORLD_SEED))
            .add_systems(
                Update,
                (update_loaded_chunks, update_camera_chunk_position).in_set(WorldSet),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSet;

fn update_camera_chunk_position(
    mut q_camera: Query<(&mut ChunkPosition, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((mut chunk_pos, transform)) = q_camera.get_single_mut() else {
        return;
    };
    let new_chunk_pos = ChunkPosition::from_world_position(&transform.translation());
    if new_chunk_pos != *chunk_pos {
        chunk_pos.0 = new_chunk_pos.0;
    }
}

fn update_loaded_chunks(
    mut commands: Commands,
    q_camera_position: Query<&GlobalTransform, (With<Camera3d>, Changed<ChunkPosition>)>,
    q_chunk_position: Query<(Entity, &ChunkPosition), With<Chunk>>,
    world_gen_noise: Res<WorldGenNoise>,
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
                .despawn_recursive();
        }
    }
    // Finally, load the new chunks
    for pos in should_be_loaded_positions {
        let chunk = generate_chunk(&world_gen_noise, &pos);
        commands.spawn((
            chunk,
            ChunkPosition(pos),
            SpatialBundle {
                transform: Transform {
                    translation: (pos * CHUNK_SIZE as i32).as_vec3(),
                    scale: Vec3::ONE * super::BLOCK_SIZE,
                    ..Default::default()
                },
                visibility: Visibility::Visible,
                ..default()
            },
        ));
    }
}

fn generate_chunk(noise: &WorldGenNoise, chunk_pos: &IVec3) -> Chunk {
    const SCALE: f64 = 60.0;
    let mut chunk_data = default::<ChunkData>();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = (noise.get([
                x as i32 + chunk_pos.x * CHUNK_SIZE as i32,
                z as i32 + chunk_pos.z * CHUNK_SIZE as i32,
            ]) * SCALE) as i32
                + 1;
            let Some(chunk_height) = Some(height - (chunk_pos.y * CHUNK_SIZE as i32))
                .filter(|h| h >= &0)
                .map(|h| h as usize)
            else {
                continue;
            };
            if chunk_height >= 1 {
                for y in (0..chunk_height - 1).filter(|h| h < &CHUNK_SIZE) {
                    *chunk_data.at_mut(x, y, z) = Block::Stone;
                }
                if chunk_height - 1 < CHUNK_SIZE {
                    *chunk_data.at_mut(x, chunk_height - 1, z) = Block::Dirt;
                }
            }
            if chunk_height < CHUNK_SIZE {
                *chunk_data.at_mut(x, chunk_height, z) = Block::Grass;
            }
        }
    }
    let blocks = Arc::new(chunk_data);
    Chunk { blocks }
}
