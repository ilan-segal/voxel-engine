use super::{spatial::SpatiallyMapped, stage::Stage, CHUNK_SIZE};
use crate::block::Block;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Clone, PartialEq)]
pub struct ChunkData {
    pub stage: Stage,
    pub blocks: Vec<Block>,
    pub noise_2d: Vec<f32>,
}

impl ChunkData {
    pub fn at(&self, x: usize, y: usize, z: usize) -> &Block {
        self.blocks.at_pos([x, y, z])
    }

    pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        let index = Self::get_array_index(x, y, z);
        self.blocks
            .get_mut(index)
            .expect("index bounds")
    }

    pub fn put(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let index = Self::get_array_index(x, y, z);
        self.blocks[index] = block;
    }

    pub fn is_meshable(&self) -> bool {
        self.blocks
            .par_iter()
            .any(|v| v.is_meshable())
    }

    pub fn get_array_index(x: usize, y: usize, z: usize) -> usize {
        CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y
    }
}
