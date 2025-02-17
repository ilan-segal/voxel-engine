use bevy::{ecs::query::QueryData, prelude::*};
use strum::IntoEnumIterator;

use crate::{
    age::Age,
    block::Block,
    item::{Item, Quantity, SECONDS_BEFORE_ELIGIBLE_FOR_PICKUP, STACK_LIMIT},
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pick_up_dropped_items)
            .add_systems(PostUpdate, clear_empty_stacks_from_inventory);
    }
}

fn clear_empty_stacks_from_inventory(mut q_inventory: Query<&mut Inventory, Changed<Inventory>>) {
    for mut inventory in q_inventory.iter_mut() {
        for index in 0..INVENTORY_WIDTH {
            let Some(slot) = inventory.hotbar[index] else {
                continue;
            };
            if slot.quantity.0 == 0 {
                inventory.hotbar[index] = None;
            }
        }
    }
}

#[derive(Component)]
pub struct PickUpRange {
    pub meters: f32,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ItemToPickUp {
    entity: Entity,
    item: &'static Item,
    quantity: &'static mut Quantity,
    transform: &'static GlobalTransform,
    age: &'static Age,
}

fn pick_up_dropped_items(
    mut commands: Commands,
    mut q_inventory: Query<(&mut Inventory, &PickUpRange, &GlobalTransform)>,
    mut q_dropped_item: Query<ItemToPickUp>,
) {
    for mut item in q_dropped_item.iter_mut() {
        if item.age.seconds < SECONDS_BEFORE_ELIGIBLE_FOR_PICKUP {
            continue;
        }
        let item_pos = item
            .transform
            .compute_transform()
            .translation;
        for (mut inventory, range, global_transform) in q_inventory.iter_mut() {
            let pos = global_transform
                .compute_transform()
                .translation;
            let distance = (item_pos - pos).length();
            if distance > range.meters {
                continue;
            }
            let Some(pickup_spec) = inventory.find_slot_to_insert_items(*item.item, *item.quantity)
            else {
                continue;
            };

            match pickup_spec.location {
                InventoryLocation::Hotbar(index) => {
                    let slot = inventory
                        .hotbar
                        .get_mut(index)
                        .expect("Valid slot index");
                    let existing_amount = (*slot)
                        .map(|s| s.quantity)
                        .unwrap_or_default();
                    let new_amount = existing_amount + pickup_spec.quantity;
                    *slot = Some(InventoryItem {
                        item: *item.item,
                        quantity: new_amount,
                    });
                }
            }

            let new_quantity = *item.quantity - pickup_spec.quantity;
            if new_quantity.0 == 0 {
                commands
                    .entity(item.entity)
                    .despawn_recursive()
            } else {
                *item.quantity = new_quantity;
            }
        }
    }
}

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
                quantity: Quantity(STACK_LIMIT),
            });
        }
        return Inventory { hotbar };
    }

    fn find_slot_to_insert_items(&self, item: Item, quantity: Quantity) -> Option<ItemPickupSpec> {
        for index in 0..INVENTORY_WIDTH {
            let Some(inventory_item) = self.hotbar[index] else {
                return Some(ItemPickupSpec {
                    location: InventoryLocation::Hotbar(index),
                    quantity,
                });
            };
            if inventory_item.item != item {
                continue;
            }
            let room = STACK_LIMIT - inventory_item.quantity.0;
            if room > 0 {
                return Some(ItemPickupSpec {
                    location: InventoryLocation::Hotbar(index),
                    quantity: Quantity(room.min(quantity.0)),
                });
            }
        }
        return None;
    }
}

struct ItemPickupSpec {
    location: InventoryLocation,
    quantity: Quantity,
}

enum InventoryLocation {
    Hotbar(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct InventoryItem {
    pub item: Item,
    pub quantity: Quantity,
}
