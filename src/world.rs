use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::HashSet;

use crate::{
    block::Block,
    chunk::{Chunk, ChunkPosition, CHUNK_SIZE},
};

const WORLD_SEED: u32 = 0xDEADBEEF;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenNoise>().add_systems(
            Update,
            (update_loaded_chunks, update_camera_chunk_position).in_set(WorldSet),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSet;

struct NoiseGenerator {
    perlin: Perlin,
    scale: f64,
    amplitude: f64,
    offset: f64,
}

impl NoiseGenerator {
    fn sample(&self, x: f64, y: f64) -> f64 {
        let sample_x = x / self.scale + self.offset;
        let sample_y = y / self.scale + self.offset;
        return self.perlin.get([sample_x, sample_y]) * self.amplitude;
    }
}

#[derive(Resource)]
struct WorldGenNoise(Vec<NoiseGenerator>);

impl WorldGenNoise {
    // Returns in range [0, 1]
    fn sample(&self, x: i32, y: i32) -> f64 {
        let mut total_sample = 0.;
        let mut total_amplitude = 0.;
        for g in &self.0 {
            total_sample += g.sample(x as f64, y as f64);
            total_amplitude += g.amplitude;
        }
        total_sample /= total_amplitude;
        0.5 + 0.5 * total_sample
    }
}

impl Default for WorldGenNoise {
    fn default() -> Self {
        Self(vec![
            NoiseGenerator {
                perlin: Perlin::new(WORLD_SEED),
                scale: 100.0,
                amplitude: 1.0,
                offset: 0.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(WORLD_SEED),
                scale: 50.0,
                amplitude: 0.5,
                offset: 10.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(WORLD_SEED),
                scale: 25.0,
                amplitude: 0.25,
                offset: 20.0,
            },
        ])
    }
}

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
    const LOAD_DISTANCE_CHUNKS: i32 = 4;
    for chunk_x in -LOAD_DISTANCE_CHUNKS..=LOAD_DISTANCE_CHUNKS {
        for chunk_y in 0..2 {
            for chunk_z in -LOAD_DISTANCE_CHUNKS..=LOAD_DISTANCE_CHUNKS {
                let cur_chunk_pos =
                    ChunkPosition(chunk_pos.0.with_y(0) + IVec3::new(chunk_x, chunk_y, chunk_z));
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
            commands.entity(entity).despawn_recursive();
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
    let mut blocks = default::<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = (noise.sample(
                x as i32 + chunk_pos.x * CHUNK_SIZE as i32,
                z as i32 + chunk_pos.z * CHUNK_SIZE as i32,
            ) * SCALE) as i32
                + 1;
            let Some(chunk_height) = Some(height - (chunk_pos.y * CHUNK_SIZE as i32))
                .filter(|h| h >= &1)
                .map(|h| h as usize)
            else {
                continue;
            };
            for y in (0..chunk_height - 1).filter(|h| h < &CHUNK_SIZE) {
                blocks[x][y][z] = Block::Stone;
            }
            if chunk_height - 1 < CHUNK_SIZE {
                blocks[x][chunk_height - 1][z] = Block::Dirt;
            }
            if chunk_height < CHUNK_SIZE {
                blocks[x][chunk_height][z] = Block::Grass;
            }
        }
    }
    Chunk { blocks }
}
