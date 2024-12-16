use bevy::utils::hashbrown::HashMap;

use crate::{block::Block, chunk::neighborhood::ChunkNeighborhood};

pub enum Structure {
    Tree { height: u32 },
}

impl Structure {
    /// Blocks and locations for those blocks within the chunk
    pub fn get_blocks(&self) -> Vec<([i32; 3], Block)> {
        match self {
            Self::Tree { height } => {
                let mut blocks = vec![];
                for i in 0..*height as i32 {
                    let pos = [0, i, 0];
                    blocks.push((pos, Block::Wood));
                }
                return blocks;
            }
        }
    }
}

pub enum StructureType {
    Tree,
}

impl StructureType {
    /// Structures and locations for those structures within the chunk neighborhood
    pub fn get_structures(&self, chunk: &ChunkNeighborhood) -> Vec<([i32; 3], Structure)> {
        todo!()
    }
}
