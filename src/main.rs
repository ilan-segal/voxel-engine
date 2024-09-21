use bevy::color::palettes::basic::{GREEN, SILVER};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

const WORLD_SEED: u32 = 0xDEADBEEF;
const CHUNK_SIZE: usize = 32;
const CUBE_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stone_material = materials.add(StandardMaterial {
        base_color: Color::from(SILVER),
        ..Default::default()
    });
    let dirt_material = materials.add(StandardMaterial {
        base_color: Color::from(Srgba::rgb(0.455, 0.278, 0.0)),
        ..Default::default()
    });
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::from(GREEN),
        ..Default::default()
    });

    let perlin_noise = Perlin::new(WORLD_SEED);
    let pos = IVec3::ZERO;
    let chunk = generate_chunk(&perlin_noise, &pos);
    commands.spawn(chunk).with_children(|child_builder| {
        // Naive mesh for every cube, very inefficient
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let maybe_material = match chunk.blocks[x][y][z] {
                        Block::Stone => Some(stone_material.clone()),
                        Block::Dirt => Some(dirt_material.clone()),
                        Block::Grass => Some(grass_material.clone()),
                        _ => None,
                    };
                    let Some(material) = maybe_material else {
                        continue;
                    };
                }
            }
        }
    });
}

fn generate_chunk(noise: &Perlin, pos: &IVec3) -> Chunk {
    let mut blocks = default::<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = (noise.get([(x as i32 + pos.x) as f64, (z as i32 + pos.z) as f64]) * 20.0)
                as usize
                + 5;
            for y in 0..height - 2 {
                blocks[x][y][z] = Block::Stone;
            }
            blocks[x][height - 1][z] = Block::Dirt;
            blocks[x][height][z] = Block::Grass;
        }
    }
    Chunk { blocks }
}

// 32x32x32 chunk of blocks
#[derive(Component, Clone, Copy)]
struct Chunk {
    // x, y, z
    blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

#[derive(Default, Clone, Copy)]
enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
}
