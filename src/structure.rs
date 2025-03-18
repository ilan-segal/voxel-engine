use crate::{
    block::Block,
    chunk::{
        data::{Blocks, Noise3d},
        CHUNK_SIZE_I32,
    },
    utils::VolumetricRange,
    world::neighborhood::Neighborhood,
};

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
    pub fn get_structures(
        &self,
        blocks: &Neighborhood<Blocks>,
        noise: &Neighborhood<Noise3d>,
    ) -> Vec<(Structure, [i32; 3])> {
        match self {
            StructureType::Tree => {
                const TREE_PROBABILITY: f32 = 0.01;
                let min = -CHUNK_SIZE_I32; // Inclusive
                let max = 2 * CHUNK_SIZE_I32; // Exclusive
                VolumetricRange::new(min..max, min..max - 1, min..max)
                    .filter(|(x, y, z)| blocks.at(*x, *y, *z).unwrap() == &Block::Grass)
                    .filter(|(x, y, z)| noise.at(*x, *y, *z).unwrap() <= &TREE_PROBABILITY)
                    .map(|(x, y, z)| {
                        let local_noise = noise.at(x, y, z).unwrap();
                        let trunk_height = if local_noise < &0.5 { 4 } else { 5 };
                        (
                            Structure::Tree {
                                trunk_height,
                                leaf_radius: 2,
                            },
                            [x, y + 1, z],
                        )
                    })
                    .collect::<_>()
            }
        }
    }
}
