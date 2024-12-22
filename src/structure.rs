use crate::{block::Block, chunk::CHUNK_SIZE_I32, world::neighborhood::ChunkNeighborhood};

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
                for x in -leaf_radius..=leaf_radius {
                    for z in -leaf_radius..=leaf_radius {
                        for y in 0.max(trunk_height - leaf_radius)..trunk_height + leaf_radius - 1 {
                            let at_trunk = x == 0 && z == 0 && y < trunk_height;
                            let at_side_edge = x.abs() == leaf_radius && z.abs() == leaf_radius;
                            let at_top_edge = (x.abs() == leaf_radius || z.abs() == leaf_radius)
                                && y == trunk_height + leaf_radius - 2;
                            if at_trunk || at_side_edge || at_top_edge {
                                continue;
                            }

                            blocks.push((Block::Leaves, [x, y, z]));
                        }
                    }
                }
                return blocks;
            }
        }
    }
}

pub enum StructureType {
    Tree,
}

impl StructureType {
    pub fn get_structures(&self, neighborhood: &ChunkNeighborhood) -> Vec<(Structure, [i32; 3])> {
        match self {
            StructureType::Tree => {
                const TREE_PROBABILITY: f32 = 0.01;
                let min = -CHUNK_SIZE_I32; // Inclusive
                let max = 2 * CHUNK_SIZE_I32; // Exclusive
                let mut trees = vec![];
                for x in min..max {
                    for z in min..max {
                        for y in min..max - 1 {
                            if neighborhood.block_at(x, y, z).unwrap() != &Block::Grass {
                                continue;
                            }
                            if neighborhood.noise_at(x, y, z).unwrap() > &TREE_PROBABILITY {
                                continue;
                            }
                            trees.push((
                                Structure::Tree {
                                    trunk_height: 4,
                                    leaf_radius: 2,
                                },
                                [x, y + 1, z],
                            ));
                        }
                    }
                }
                return trees;
            }
        }
    }
}
