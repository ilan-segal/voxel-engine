use super::{stage::Stage, CHUNK_ARRAY_SIZE, CHUNK_SIZE};
use crate::block::Block;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Clone, Copy)]
pub struct ChunkData {
    pub stage: Stage,
    pub blocks: [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
    pub noise_2d: [f32; CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::filled(Block::default())
    }
}

impl ChunkData {
    pub fn filled(block: Block) -> Self {
        Self {
            blocks: [block; CHUNK_ARRAY_SIZE],
            stage: Stage::Terrain,
            noise_2d: [0.; CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    pub fn at(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Self::get_array_index(x, y, z)]
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
            .into_par_iter()
            .any(|v| v.is_meshable())
    }

    pub fn get_array_index(x: usize, y: usize, z: usize) -> usize {
        CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y
    }
}
