use crate::block::Block;
use bevy::{prelude::*, utils::HashMap};

pub const CHUNK_SIZE: usize = 32;

pub type LayerMask = [u32; CHUNK_SIZE];
pub type ChunkMask = [LayerMask; CHUNK_SIZE];

pub trait BitWiseOps {
    fn or(&self, other: &Self) -> Self;
    fn and(&self, other: &Self) -> Self;
}

impl BitWiseOps for LayerMask {
    fn or(&self, other: &Self) -> Self {
        std::array::from_fn(|i| self[i] | other[i])
    }
    fn and(&self, other: &Self) -> Self {
        std::array::from_fn(|i| self[i] & other[i])
    }
}

impl BitWiseOps for ChunkMask {
    fn or(&self, other: &Self) -> Self {
        std::array::from_fn(|i| self[i].or(&other[i]))
    }
    fn and(&self, other: &Self) -> Self {
        std::array::from_fn(|i| self[i].and(&other[i]))
    }
}

pub trait VoxelMask {
    fn set_bit(&mut self, x: usize, y: usize, z: usize, value: bool);
}

impl VoxelMask for ChunkMask {
    fn set_bit(&mut self, x: usize, y: usize, z: usize, value: bool) {
        if value {
            self[y][z] |= 1 << x;
        } else {
            self[y][z] &= (!1) << x;
        }
    }
}

// 32x32x32 chunk of blocks
#[derive(Component, Clone)]
pub struct Chunk {
    // x, y, z
    pub masks: HashMap<Block, ChunkMask>,
}

impl Chunk {
    pub fn get_opacity_mask(&self) -> ChunkMask {
        self.masks
            .iter()
            .filter_map(|(block, mask)| {
                if block == &Block::Air {
                    None
                } else {
                    Some(*mask)
                }
            })
            .reduce(|a, b| a.or(&b))
            .unwrap()
    }
}

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
