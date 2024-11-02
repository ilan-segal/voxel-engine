use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::block::Block;

pub const CHUNK_SIZE: usize = 32;
const CHUNK_ARRAY_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy)]
pub struct ChunkData([Block; CHUNK_ARRAY_SIZE]);

impl Default for ChunkData {
    fn default() -> Self {
        Self::filled(Block::default())
    }
}

impl ChunkData {
    pub fn filled(block: Block) -> Self {
        Self([block; CHUNK_ARRAY_SIZE])
    }

    pub fn at(&self, x: usize, y: usize, z: usize) -> Block {
        self.0[Self::get_array_index(x, y, z)]
    }

    pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        let index = Self::get_array_index(x, y, z);
        self.0
            .get_mut(index)
            .expect("index bounds")
    }

    pub fn is_meshable(&self) -> bool {
        self.0
            .into_par_iter()
            .any(|v| v.is_meshable())
    }

    fn get_array_index(x: usize, y: usize, z: usize) -> usize {
        CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y
    }
}
