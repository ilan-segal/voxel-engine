use std::sync::RwLockReadGuard;

use crate::{
    block::Block,
    chunk::{data::ChunkData, CHUNK_SIZE},
    file_io::block_spec::{BlockPositions, BlockSpec},
};
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum ChunkFileData {
    Version1 { specs: Vec<BlockSpec> },
}

impl<'a> From<RwLockReadGuard<'a, ChunkData>> for ChunkFileData {
    fn from(value: RwLockReadGuard<'a, ChunkData>) -> Self {
        const SKIPPED_BLOCK: Block = Block::Air;
        let mut block_positions: HashMap<Block, _> = HashMap::new();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let cur_block = value.at(x, y, z);
                    if cur_block == SKIPPED_BLOCK {
                        continue;
                    }
                    block_positions
                        .entry_ref(&cur_block)
                        .or_insert_with(|| vec![])
                        .push([x as u8, y as u8, z as u8]);
                }
            }
        }
        let specs = block_positions
            .iter()
            .map(|(block, positions)| BlockSpec {
                block: *block,
                positions: BlockPositions::from(positions),
            })
            .collect::<_>();
        return Self::Version1 { specs };
    }
}

impl From<ChunkFileData> for ChunkData {
    fn from(data: ChunkFileData) -> Self {
        match data {
            ChunkFileData::Version1 { specs } => parse_chunk_from_specs(specs),
        }
    }
}

fn parse_chunk_from_specs(specs: Vec<BlockSpec>) -> ChunkData {
    let mut data = ChunkData::default();
    for spec in specs {
        match spec.positions {
            BlockPositions::OneByOne { positions } => {
                for [x, y, z] in positions {
                    let x = x as usize;
                    let y = y as usize;
                    let z = z as usize;
                    data.put(x, y, z, spec.block);
                }
            }
            BlockPositions::Mask { .. } => todo!(),
        }
    }
    return data;
}
