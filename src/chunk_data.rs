use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::block::Block;

pub const CHUNK_SIZE: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct ChunkData([[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

impl ChunkData {
    pub fn at(&self, x: usize, y: usize, z: usize) -> Block {
        self.0[x][y][z]
    }

    pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        self.0
            .get_mut(x)
            .expect("x bounds")
            .get_mut(y)
            .expect("y bounds")
            .get_mut(z)
            .expect("z bounds")
    }

    pub fn is_meshable(&self) -> bool {
        self.0
            .into_par_iter()
            .flatten()
            .flatten()
            .any(|v| v.is_meshable())
    }
}
