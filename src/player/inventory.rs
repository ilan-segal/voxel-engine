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

enum HotbarChange {
    Up,
    Down,
}

impl HotbarSelection {
    pub fn increase(&mut self) {
        self.change(HotbarChange::Up);
    }

    pub fn decrease(&mut self) {
        self.change(HotbarChange::Down);
    }

    fn change(&mut self, change: HotbarChange) {
        self.index = match change {
            HotbarChange::Down => {
                if self.index == 0 {
                    INVENTORY_WIDTH as u8 - 1
                } else {
                    self.index - 1
                }
            }
            HotbarChange::Up => {
                if self.index == INVENTORY_WIDTH as u8 - 1 {
                    0
                } else {
                    self.index + 1
                }
            }
        }
    }
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
