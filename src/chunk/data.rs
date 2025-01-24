use super::{spatial::SpatiallyMapped, CHUNK_SIZE};
use crate::block::Block;
use bevy::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

// #[derive(Clone, PartialEq)]
// pub struct ChunkData {
//     pub stage: Stage,
//     pub blocks: Vec<Block>,
//     pub perlin_2d: Vec<f32>,
// }

#[derive(Component, Clone)]
pub struct Blocks(pub Vec<Block>);

impl Default for Blocks {
    fn default() -> Self {
        let blocks = std::iter::repeat_n(Block::default(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
            .collect::<_>();
        return Self(blocks);
    }
}

impl SpatiallyMapped<3> for Blocks {
    type Item = Block;

    fn at_pos(&self, pos: [usize; 3]) -> &Block {
        self.0.at_pos(pos)
    }

    fn at_pos_mut(&mut self, pos: [usize; 3]) -> &mut Block {
        self.0.at_pos_mut(pos)
    }
}

impl Blocks {
    pub fn is_meshable(&self) -> bool {
        self.0
            .par_iter()
            .any(|b| b.is_meshable())
    }
}

#[derive(Component, Clone)]
pub struct Perlin2d(pub Vec<f32>);

impl SpatiallyMapped<2> for Perlin2d {
    type Item = f32;

    fn at_pos(&self, pos: [usize; 2]) -> &f32 {
        self.0.at_pos(pos)
    }

    fn at_pos_mut(&mut self, pos: [usize; 2]) -> &mut f32 {
        self.0.at_pos_mut(pos)
    }
}

#[derive(Component, Clone)]
pub struct Noise3d(pub Vec<f32>);

impl SpatiallyMapped<3> for Noise3d {
    type Item = f32;

    fn at_pos(&self, pos: [usize; 3]) -> &f32 {
        self.0.at_pos(pos)
    }

    fn at_pos_mut(&mut self, pos: [usize; 3]) -> &mut f32 {
        self.0.at_pos_mut(pos)
    }
}

// impl ChunkData {
//     pub fn at(&self, x: usize, y: usize, z: usize) -> &Block {
//         self.blocks.at_pos([x, y, z])
//     }

//     pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
//         let index = Self::get_array_index(x, y, z);
//         self.blocks
//             .get_mut(index)
//             .expect("index bounds")
//     }

//     pub fn put(&mut self, x: usize, y: usize, z: usize, block: Block) {
//         let index = Self::get_array_index(x, y, z);
//         self.blocks[index] = block;
//     }

//     pub fn is_meshable(&self) -> bool {
//         self.blocks
//             .par_iter()
//             .any(|v| v.is_meshable())
//     }

//     pub fn get_array_index(x: usize, y: usize, z: usize) -> usize {
//         CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y
//     }
// }
