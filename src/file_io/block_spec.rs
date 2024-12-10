use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    chunk::data::{ChunkData, CHUNK_ARRAY_SIZE},
};

#[derive(Serialize, Deserialize)]
pub struct BlockSpec {
    pub block: Block,
    pub positions: BlockPositions,
}

type MaskItem = u128;
const MASK_ITEM_SIZE: usize = size_of::<MaskItem>();

#[derive(Serialize, Deserialize)]
#[serde(tag = "representation")]
pub enum BlockPositions {
    OneByOne { positions: Vec<[u8; 3]> },
    Mask { mask: Vec<MaskItem> },
}

impl From<&Vec<[u8; 3]>> for BlockPositions {
    fn from(value: &Vec<[u8; 3]>) -> Self {
        const MAXIMUM_VALUES_FOR_ONE_BY_ONE: usize = 10;
        if value.len() <= MAXIMUM_VALUES_FOR_ONE_BY_ONE {
            return Self::OneByOne {
                positions: value.clone(),
            };
        }
        let mut mask =
            std::iter::repeat_n(0, CHUNK_ARRAY_SIZE / MASK_ITEM_SIZE).collect::<Vec<_>>();
        for [x, y, z] in value {
            let x = *x as usize;
            let y = *y as usize;
            let z = *z as usize;
            let array_index = ChunkData::get_array_index(x, y, z);
            let position_in_mask = array_index / MASK_ITEM_SIZE;
            let bit = array_index % MASK_ITEM_SIZE;
            mask[position_in_mask] |= 1 << bit;
        }
        return Self::Mask { mask };
    }
}
