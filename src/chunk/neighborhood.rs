use super::{data::ChunkData, layer_to_xyz, CHUNK_SIZE};
use crate::block::{Block, BlockSide};
use std::sync::{Arc, RwLock};

/// Represents a 3x3x3 cube of chunks
pub struct ChunkNeighborhood {
    pub chunks: [[[Option<Arc<RwLock<ChunkData>>>; 3]; 3]; 3],
}

impl ChunkNeighborhood {
    pub fn at(&self, x: i32, y: i32, z: i32) -> Block {
        fn get_chunk_pos_coord(in_chunk_coord: i32) -> (usize, usize) {
            if in_chunk_coord < 0 {
                ((in_chunk_coord + CHUNK_SIZE as i32) as usize, 0)
            } else if in_chunk_coord < CHUNK_SIZE as i32 {
                (in_chunk_coord as usize, 1)
            } else {
                ((in_chunk_coord - CHUNK_SIZE as i32) as usize, 2)
            }
        }
        let (x, chunk_x) = get_chunk_pos_coord(x);
        let (y, chunk_y) = get_chunk_pos_coord(y);
        let (z, chunk_z) = get_chunk_pos_coord(z);

        return self.chunks[chunk_x][chunk_y][chunk_z]
            .as_deref()
            .map(|data| *data.read().unwrap().at(x, y, z))
            .unwrap_or_default();
    }

    pub fn at_layer(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> Block {
        let (x, y, z) = layer_to_xyz(side, layer, row, col);
        self.at(x, y, z)
    }

    pub fn middle(&self) -> Arc<RwLock<ChunkData>> {
        self.chunks[1][1][1].clone().unwrap()
    }

    pub fn block_is_hidden_from_above(
        &self,
        side: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> bool {
        self.at_layer(side, layer + 1, row, col) != Block::Air
    }

    pub fn count_block(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> u8 {
        if self.at_layer(side, layer, row, col) == Block::Air {
            0
        } else {
            1
        }
    }
}
