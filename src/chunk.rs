use std::sync::Arc;

use crate::block::Block;
use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 32;

pub type ChunkData = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

// 32x32x32 chunk of blocks
#[derive(Component, Clone)]
pub struct Chunk {
    // x, y, z
    pub blocks: Arc<ChunkData>,
}

#[derive(Component, PartialEq, Eq, Default, Hash, Clone, Copy)]
pub struct ChunkPosition(pub IVec3);

impl ChunkPosition {
    pub fn from_world_position(p: &Vec3) -> Self {
        ChunkPosition((*p / (CHUNK_SIZE as f32)).floor().as_ivec3())
    }
}
