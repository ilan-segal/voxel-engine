use super::{block_icons::BlockIconMaterials, health::HealthDisplayRoot, Ui, UiFont};
use crate::{
    item::Quantity,
    player::inventory::{HotbarSelection, Inventory, ItemType, HOTBAR_SIZE},
    state::GameState,
};
use bevy::prelude::*;

pub struct HotbarUiPlugin;

impl Plugin for HotbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (update_selected_slot, update_item_display).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::InGame), spawn_hotbar)
            .observe(add_slots);
    }
}

#[derive(Component)]
pub struct HotbarDisplayRoot;

#[derive(Resource)]
struct HotbarSprites {
    slot: UiImage,
    selected_slot: UiImage,
}

#[derive(Component)]
struct HotbarIndex(usize);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprites = HotbarSprites {
        slot: UiImage::new(asset_server.load("ui/hud/inventory/slot.png")),
        selected_slot: UiImage::new(asset_server.load("ui/hud/inventory/selected_slot.png")),
    };
    commands.insert_resource(sprites);
}

const SLOT_SPRITE_SIZE: f32 = 50.0;

#[derive(Component)]
struct UiHotbar;

fn spawn_hotbar(mut commands: Commands) {
    commands
        .spawn((
            Ui,
            UiHotbar,
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::End,
                    justify_self: JustifySelf::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((Ui, HealthDisplayRoot, NodeBundle::default()));
            builder.spawn((
                Ui,
                HotbarDisplayRoot,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        align_content: AlignContent::Start,
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

#[derive(Component)]
struct HotbarSlot;

fn add_slots(
    trigger: Trigger<OnAdd, HotbarDisplayRoot>,
    mut commands: Commands,
    sprites: Res<HotbarSprites>,
) {
    let entity = trigger.entity();
    let mut entity_commands = commands
        .get_entity(entity)
        .expect("Triggered hotbar root");
    entity_commands.with_children(|builder| {
        let width = Val::Px(SLOT_SPRITE_SIZE);
        let height = Val::Px(SLOT_SPRITE_SIZE);
        let margin = UiRect::all(Val::Px(1.0));
        for i in 0..HOTBAR_SIZE {
            builder.spawn((
                Ui,
                HotbarSlot,
                HotbarIndex(i),
                sprites.slot.clone(),
                NodeBundle {
                    style: Style {
                        width,
                        height,
                        margin,
                        ..default()
                    },
                    ..default()
                },
            ));
        }
    });
}

fn update_selected_slot(
    selection: Query<&HotbarSelection, Changed<HotbarSelection>>,
    mut hotbar_display: Query<(&mut UiImage, &HotbarIndex), With<HotbarSlot>>,
    sprites: Res<HotbarSprites>,
) {
    let Ok(HotbarSelection { index }) = selection.get_single() else {
        return;
    };
    for (mut image, hotbar_index) in hotbar_display.iter_mut() {
        *image = if hotbar_index.0 as u8 == *index {
            sprites.selected_slot.clone()
        } else {
            sprites.slot.clone()
        };
    }
}

fn update_item_display(
    mut commands: Commands,
    q_inventory: Query<&Inventory, Changed<Inventory>>,
    q_hotbar_display: Query<(Entity, &HotbarIndex), With<HotbarSlot>>,
    block_icons: Res<BlockIconMaterials>,
    font: Res<UiFont>,
) {
    let Ok(inventory) = q_inventory.get_single() else {
        return;
    };
    for (entity, HotbarIndex(index)) in q_hotbar_display.iter() {
        commands
            .entity(entity)
            .despawn_descendants();
        let Some(item) = inventory.hotbar[*index] else {
            continue;
        };
        let material = match item.item {
            ItemType::Block(block) => block_icons
                .map
                .get(&block)
                .expect("Block should have a material for icon"),
        };
        let quantity_font_size = match item.quantity {
            Quantity::Infinity => 24.0,
            Quantity::Finite(..) => 18.0,
        };
        let item_icon_id = commands
            .spawn((
                Ui,
                HotbarIndex(*index),
                UiImage::new(material.clone_weak()),
                NodeBundle {
                    style: Style {
                        width: Val::Px(SLOT_SPRITE_SIZE),
                        height: Val::Px(SLOT_SPRITE_SIZE),
                        justify_content: JustifyContent::End,
                        align_content: AlignContent::FlexEnd,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|builder| {
                builder.spawn((
                    Ui,
                    TextBundle::from_section(
                        item.quantity,
                        TextStyle {
                            font: font.0.clone_weak(),
                            font_size: quantity_font_size,
                            color: Color::WHITE,
                        },
                    )
                    .with_text_justify(JustifyText::Right)
                    .with_style(Style {
                        /*
                        Sorry for the ugly equation, it just calculates a good position
                        for the text.
                            Font size -> Top offset as a percentage of SLOT_SPRITE_SIZE
                            18.0 -> 0.55
                            24.0 -> 0.5
                         */
                        top: Val::Px(SLOT_SPRITE_SIZE * (quantity_font_size / -120.0 + 0.7)),
                        right: Val::Px(8.0),
                        ..default()
                    }),
                ));
            })
            .id();
        commands
            .entity(entity)
            .push_children(&[item_icon_id]);
    }
}
