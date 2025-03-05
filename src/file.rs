use std::{collections::HashMap, ops::Range};

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    chunk::{data::Blocks, spatial::SpatiallyMapped, CHUNK_SIZE},
    utils::VolumetricRange,
    world::seed::WorldSeed,
};

#[derive(Serialize, Deserialize)]
pub struct WorldData {
    seed: WorldSeed,
}

#[derive(Serialize, Deserialize)]
pub struct BlockMasks {
    map: HashMap<Block, BlockMask>,
}

impl From<&Blocks> for BlockMasks {
    fn from(blocks: &Blocks) -> Self {
        let mut map = HashMap::new();
        for block in blocks.0.iter().unique() {
            let mask = BlockMask::from(blocks, block);
            map.insert(*block, mask);
        }
        return BlockMasks { map };
    }
}

impl From<BlockMasks> for Blocks {
    fn from(value: BlockMasks) -> Self {
        let mut blocks = Blocks::default();
        const RANGE: Range<usize> = 0..CHUNK_SIZE;
        for (block, mask) in value.map.iter() {
            for (x, y, z) in VolumetricRange::new(RANGE, RANGE, RANGE) {
                if mask.get_at(x, y, z) {
                    *blocks.at_pos_mut([x, y, z]) = *block;
                }
            }
        }
        return blocks;
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockMask {
    rows: [[u32; CHUNK_SIZE]; CHUNK_SIZE],
}

impl BlockMask {
    fn from(blocks: &Blocks, block: &Block) -> Self {
        let blocks = core::array::from_fn(|x| encode_layer(blocks, block, x));
        Self { rows: blocks }
    }

    fn get_at(&self, x: usize, y: usize, z: usize) -> bool {
        self.rows[x][y] >> z & 1 == 1
    }
}

fn encode_layer(blocks: &Blocks, block: &Block, x: usize) -> [u32; CHUNK_SIZE] {
    core::array::from_fn(|y| encode_row(blocks, block, x, y))
}

fn encode_row(blocks: &Blocks, block: &Block, x: usize, y: usize) -> u32 {
    (0..CHUNK_SIZE)
        .into_par_iter()
        .filter_map(|z| {
            if blocks.at_pos([x, y, z]) == block {
                Some(1 << z)
            } else {
                None
            }
        })
        .reduce(|| 0, |a, b| a | b)
}

// pub struct PlayerData {
//     world_position: [i32; 3],
//     inventory: Inventory,
//     health: Health,
// }
