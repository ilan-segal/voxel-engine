use crate::{block::BlockSide, chunk::data::ChunkData};
use bevy::prelude::*;
use std::sync::Arc;

pub mod data;
pub mod index;
pub mod neighborhood;
pub mod position;

pub const CHUNK_SIZE: usize = 32;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(index::ChunkIndexPlugin);
    }
}

// 32x32x32 chunk of blocks
#[derive(Component, Clone)]
pub struct Chunk {
    // x, y, z
    pub blocks: Arc<ChunkData>,
}

impl Chunk {
    pub fn new(data: ChunkData) -> Self {
        Self {
            blocks: Arc::new(data),
        }
    }
}

pub fn layer_to_xyz(side: &BlockSide, layer: i32, row: i32, col: i32) -> (i32, i32, i32) {
    match side {
        BlockSide::Up => (row, layer, col),
        BlockSide::Down => (row, CHUNK_SIZE as i32 - 1 - layer, col),
        BlockSide::North => (layer, col, row),
        BlockSide::South => (CHUNK_SIZE as i32 - 1 - layer, row, col),
        BlockSide::East => (col, row, layer),
        BlockSide::West => (row, col, CHUNK_SIZE as i32 - 1 - layer),
    }
}
