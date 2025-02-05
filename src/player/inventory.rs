use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::{block::Block, item::Quantity};

pub const HOTBAR_SIZE: usize = 10;

#[derive(Component, Default)]
pub struct HotbarSelection {
    pub index: u8,
}

#[derive(Component, Default, Clone, Copy)]
pub struct Inventory {
    pub hotbar: [Option<InventoryItem>; HOTBAR_SIZE],
}

impl Inventory {
    pub fn creative_default() -> Self {
        let mut hotbar = [const { None }; HOTBAR_SIZE];
        for (i, block) in Block::iter()
            .filter(|block| block != &Block::Air)
            .enumerate()
            .take(HOTBAR_SIZE)
        {
            hotbar[i] = Some(InventoryItem {
                item: ItemType::Block(block),
                quantity: Quantity::Infinity,
            });
        }
        return Inventory { hotbar };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InventoryItem {
    pub item: ItemType,
    pub quantity: Quantity,
}

#[derive(Clone, Copy, Debug)]
pub enum ItemType {
    Block(Block),
    // TODO: Add variants for non-block items (e.g. food)
}

impl From<Block> for ItemType {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}
