use itertools::Itertools;

use crate::{
    block::Block,
    chunk::{
        data::{Blocks, Noise3d},
        spatial::SpatiallyMapped,
        CHUNK_SIZE, CHUNK_SIZE_I32, CHUNK_SIZE_U32,
    },
    utils::{fast_hash, fast_hash_f32, VolumetricRange},
    world::neighborhood::Neighborhood,
};

#[derive(Clone)]
pub enum Structure {
    Tree { trunk_height: u8, leaf_radius: u8 },
}

impl Structure {
    pub fn get_blocks(&self) -> Vec<(Block, [i32; 3])> {
        match self {
            Structure::Tree {
                trunk_height,
                leaf_radius,
            } => {
                let trunk_height = *trunk_height as i32;
                let leaf_radius = *leaf_radius as i32;
                let mut blocks = vec![];
                for y in 0..trunk_height {
                    blocks.push((Block::Wood, [0, y, 0]));
                }
                // Leaves
                VolumetricRange::new(
                    -leaf_radius..leaf_radius + 1,
                    0.max(trunk_height - leaf_radius)..trunk_height + leaf_radius - 1,
                    -leaf_radius..leaf_radius + 1,
                )
                .filter(|(x, y, z)| x != &0 || z != &0 || y >= &trunk_height)
                .filter(|(x, _, z)| x.abs() != leaf_radius || z.abs() != leaf_radius)
                .filter(|(x, y, z)| {
                    (x.abs() != leaf_radius && z.abs() != leaf_radius)
                        || *y != trunk_height + leaf_radius - 2
                })
                .for_each(|(x, y, z)| {
                    blocks.push((Block::Leaves, [x, y, z]));
                });
                return blocks;
            }
        }
    }
}

pub enum StructureType {
    Tree,
}

impl StructureType {
    pub fn get_structure_blocks(
        &self,
        blocks: &Neighborhood<Blocks>,
        noise: &Neighborhood<Noise3d>,
    ) -> Vec<(Block, [usize; 3])> {
        self.get_structures(blocks, noise)
            .flat_map(|(structure, [x0, y0, z0])| {
                structure
                    .get_blocks()
                    .iter()
                    .map(|(block, [x, y, z])| {
                        let x = x0 + x;
                        let y = y0 + y;
                        let z = z0 + z;
                        (*block, [x, y, z])
                    })
                    .collect::<Vec<_>>()
            })
            .filter_map(|(block, [x, y, z])| {
                let x = usize::try_from(x).ok()?;
                let y = usize::try_from(y).ok()?;
                let z = usize::try_from(z).ok()?;
                if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
                    return None;
                }
                return Some((block, [x, y, z]));
            })
            .collect::<_>()
    }

    fn get_structures<'a>(
        &self,
        blocks: &'a Neighborhood<Blocks>,
        noise: &'a Neighborhood<Noise3d>,
    ) -> impl Iterator<Item = (Structure, [i32; 3])> + use<'a> {
        match self {
            StructureType::Tree => {
                const NUM_TREES_PER_CHUNK: usize = 20;
                let tree = Structure::Tree {
                    trunk_height: 4,
                    leaf_radius: 2,
                };
                let range = -1..=1;
                let chunk_positions = range
                    .clone()
                    .cartesian_product(range.clone())
                    .cartesian_product(range.clone());
                chunk_positions
                    .map(|((x, y), z)| (x, y, z))
                    .flat_map(|(x_chunk, y_chunk, z_chunk)| {
                        let blocks = blocks
                            .get_chunk(x_chunk, y_chunk, z_chunk)
                            .clone()
                            .expect("Full block neighborhood");
                        let noise = noise
                            .get_chunk(x_chunk, y_chunk, z_chunk)
                            .clone()
                            .expect("Full noise neighborhood");
                        return generate_tree_spots_xyz(
                            NUM_TREES_PER_CHUNK,
                            noise.as_ref(),
                            blocks.as_ref(),
                        )
                        .map(move |[x_local, y_local, z_local]| {
                            [
                                x_local as i32 + CHUNK_SIZE_I32 * x_chunk,
                                y_local as i32 + CHUNK_SIZE_I32 * y_chunk,
                                z_local as i32 + CHUNK_SIZE_I32 * z_chunk,
                            ]
                        })
                        .collect::<Vec<_>>();
                    })
                    .map(move |pos| (tree.clone(), pos))
            }
        }
    }
}

fn generate_tree_spots_xyz<'a>(
    count: usize,
    noise: &'a Noise3d,
    blocks: &'a Blocks,
) -> impl Iterator<Item = [u32; 3]> + use<'a> {
    generate_tree_spots_xz(noise)
        .take(count)
        .filter_map(|xz| find_grass_block(xz, blocks))
        .map(|[x, y, z]| [x, y + 1, z])
}

fn generate_tree_spots_xz(noise: &Noise3d) -> impl Iterator<Item = [u32; 2]> + use<'_> {
    noise
        .0
        .iter()
        .map(|x| fast_hash_f32(*x))
        .map(|x| {
            let z = fast_hash(x);
            [x % CHUNK_SIZE_U32, z % CHUNK_SIZE_U32]
        })
}

fn find_grass_block([x, z]: [u32; 2], blocks: &Blocks) -> Option<[u32; 3]> {
    (0..=CHUNK_SIZE_U32 - 1)
        .rev()
        .filter(|y| blocks.at_pos([x as usize, *y as usize, z as usize]) == &Block::Grass)
        .next()
        .map(|y| [x, y, z])
}
