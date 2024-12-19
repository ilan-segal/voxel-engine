use super::{data::ChunkBundle, layer_to_xyz, CHUNK_SIZE};
use crate::{
    block::{Block, BlockSide},
    chunk::spatial::SpatiallyMapped,
};
use std::sync::Arc;

/// Represents a 3x3x3 cube of chunks
pub struct ChunkNeighborhood {
    pub chunks: [[[Option<Arc<ChunkBundle>>; 3]; 3]; 3],
}

impl ChunkNeighborhood {
    pub fn at(&self, x: i32, y: i32, z: i32) -> Option<&Block> {
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

        self.chunks[chunk_x][chunk_y][chunk_z]
            .as_ref()
            .map(|bundle| bundle.blocks.at_pos([x, y, z]))
    }

    pub fn at_layer(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> Option<&Block> {
        let (x, y, z) = layer_to_xyz(side, layer, row, col);
        self.at(x, y, z)
    }

    pub fn middle(&self) -> Option<Arc<ChunkBundle>> {
        self.chunks[1][1][1].clone()
    }

    pub fn block_is_hidden_from_above(
        &self,
        side: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> bool {
        match self.at_layer(side, layer + 1, row, col) {
            None | Some(&Block::Air) => false,
            _ => true,
        }
    }

    pub fn count_block(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> u8 {
        match self.at_layer(side, layer, row, col) {
            None | Some(&Block::Air) => 0,
            _ => 1,
        }
    }
}
