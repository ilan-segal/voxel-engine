use bevy::prelude::*;

use crate::chunk::CHUNK_SIZE;

#[derive(Component, PartialEq, Eq, Default, Hash, Clone, Copy)]
pub struct ChunkPosition(pub IVec3);

impl ChunkPosition {
    pub fn from_world_position(p: &Vec3) -> Self {
        ChunkPosition(
            (*p / (CHUNK_SIZE as f32))
                .floor()
                .as_ivec3(),
        )
    }
}
