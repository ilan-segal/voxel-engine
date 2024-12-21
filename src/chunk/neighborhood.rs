use super::{data::ChunkBundle, layer_to_xyz, stage::Stage, CHUNK_SIZE};
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
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Option<&Block> {
        let (x, chunk_x, y, chunk_y, z, chunk_z) = to_local_coordinates(x, y, z);

        self.chunks[chunk_x][chunk_y][chunk_z]
            .as_ref()
            .map(|bundle| bundle.blocks.at_pos([x, y, z]))
    }

    pub fn noise_at(&self, x: i32, y: i32, z: i32) -> Option<&f32> {
        let (x, chunk_x, y, chunk_y, z, chunk_z) = to_local_coordinates(x, y, z);

        self.chunks[chunk_x][chunk_y][chunk_z]
            .as_ref()
            .map(|bundle| bundle.noise_3d.at_pos([x, y, z]))
    }

    pub fn at_layer(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> Option<&Block> {
        let (x, y, z) = layer_to_xyz(side, layer, row, col);
        self.block_at(x, y, z)
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

    pub fn get_lowest_stage(&self) -> Stage {
        self.iter_chunks()
            .filter_map(|chunk| chunk.clone())
            .map(|chunk| chunk.stage)
            .min()
            .unwrap_or_default()
    }

    pub fn iter_chunks(&self) -> impl Iterator<Item = &Option<Arc<ChunkBundle>>> {
        self.chunks.iter().flatten().flatten()
    }
}

fn to_local_coordinates(x: i32, y: i32, z: i32) -> (usize, usize, usize, usize, usize, usize) {
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
    (x, chunk_x, y, chunk_y, z, chunk_z)
}
