use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    block::Block,
    item::{Item, Quantity},
};

pub const INVENTORY_WIDTH: usize = 10;

#[derive(Component, Default)]
pub struct HotbarSelection {
    pub index: u8,
}

#[derive(Component, Default, Clone, Copy)]
pub struct Inventory {
    pub hotbar: [Option<InventoryItem>; INVENTORY_WIDTH],
}

impl Inventory {
    pub fn creative_default() -> Self {
        let mut hotbar = [const { None }; INVENTORY_WIDTH];
        for (i, block) in Block::iter()
            .filter(|block| block != &Block::Air)
            .enumerate()
            .take(INVENTORY_WIDTH)
        {
            hotbar[i] = Some(InventoryItem {
                item: Item::Block(block),
                quantity: Quantity::Infinity,
            });
        }
        return Inventory { hotbar };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InventoryItem {
    pub item: Item,
    pub quantity: Quantity,
}

#[derive(Component)]
pub struct InventorySlot;

type InventoryRow = [Option<Entity>; INVENTORY_WIDTH];

#[derive(Component)]
pub struct InventoryIndex {
    inventory: Vec<InventoryRow>,
    hotbar: InventoryRow,
}
