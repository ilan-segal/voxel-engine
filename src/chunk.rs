use crate::block::Block;
use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 32;

// 32x32x32 chunk of blocks
#[derive(Component, Clone, Copy)]
pub struct Chunk {
    // x, y, z
    pub blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

#[derive(Component, PartialEq, Eq, Default)]
pub struct ChunkPosition(pub IVec3);

impl ChunkPosition {
    pub fn from_world_position(p: &Vec3) -> Self {
        ChunkPosition((*p / (CHUNK_SIZE as f32)).floor().as_ivec3())
    }
}
