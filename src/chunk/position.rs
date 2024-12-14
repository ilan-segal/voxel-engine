use crate::chunk::CHUNK_SIZE;
use bevy::prelude::*;

#[derive(Component, PartialEq, Eq, Default, Hash, Clone, Copy, Debug)]
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
