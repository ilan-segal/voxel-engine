use crate::block::Block;
use bevy::{prelude::*, utils::HashMap};

pub const CHUNK_SIZE: usize = 32;

type LayerMask = [u32; CHUNK_SIZE];
type ChunkMask = [LayerMask; CHUNK_SIZE];

// 32x32x32 chunk of blocks
#[derive(Component, Clone)]
pub struct Chunk {
    // x, y, z
    pub blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub masks: HashMap<Block, ChunkMask>,
}

impl Chunk {
    pub fn new(blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]) -> Self {
        let mut masks = HashMap::default();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = blocks[x][y][z];
                    let mask = masks
                        .entry(block)
                        .or_insert_with(ChunkMask::default);
                    mask[y][z] |= 1 << x;
                }
            }
        }
        Self { blocks, masks }
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
