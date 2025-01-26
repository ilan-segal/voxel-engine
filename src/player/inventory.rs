use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::block::Block;

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
            .enumerate()
            .take(HOTBAR_SIZE)
        {
            hotbar[i] = Some(InventoryItem::infinite(block.into()));
        }
        return Inventory { hotbar };
    }
}

#[derive(Clone, Copy)]
pub struct InventoryItem {
    item: ItemType,
    quantity: ItemQuantity,
}

impl InventoryItem {
    pub fn infinite(item: ItemType) -> Self {
        Self {
            item,
            quantity: ItemQuantity::Infinity,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ItemType {
    Block(Block),
    // TODO: Add variants for non-block items (e.g. food)
}

impl From<Block> for ItemType {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}

#[derive(Clone, Copy)]
pub enum ItemQuantity {
    Infinity,
    Number(u8),
}
