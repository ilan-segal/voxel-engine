use super::{block_icons::BlockIconMaterials, health::HealthDisplayRoot, Ui, UiFont};
use crate::{
    item::Item,
    player::inventory::{HotbarSelection, Inventory, INVENTORY_WIDTH},
    state::AppState,
    ui::HudUi,
};
use bevy::prelude::*;

pub struct HotbarUiPlugin;

impl Plugin for HotbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (update_selected_slot, update_item_display).run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::InGame), spawn_hotbar)
            .add_observer(add_slots);
    }
}

#[derive(Component)]
#[require(HudUi)]
pub struct HotbarDisplayRoot;

#[derive(Resource)]
struct HotbarSprites {
    slot: Handle<Image>,
    selected_slot: Handle<Image>,
}

#[derive(Component)]
struct HotbarIndex(usize);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprites = HotbarSprites {
        slot: asset_server.load("ui/hud/inventory/slot.png"),
        selected_slot: asset_server.load("ui/hud/inventory/selected_slot.png"),
    };
    commands.insert_resource(sprites);
}

const SLOT_SPRITE_SIZE: f32 = 50.0;

#[derive(Component)]
#[require(HudUi)]
struct UiHotbar;

fn spawn_hotbar(mut commands: Commands) {
    commands
        .spawn((
            UiHotbar,
            Node {
                height: Val::Percent(100.0),
                justify_content: JustifyContent::End,
                justify_self: JustifySelf::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((HealthDisplayRoot, Node::default()));
            builder.spawn((
                HotbarDisplayRoot,
                Node {
                    width: Val::Percent(100.0),
                    align_content: AlignContent::Start,
                    ..default()
                },
            ));
        });
}

#[derive(Component)]
#[require(HudUi)]
struct HotbarSlot;

fn add_slots(
    trigger: Trigger<OnAdd, HotbarDisplayRoot>,
    mut commands: Commands,
    sprites: Res<HotbarSprites>,
) {
    let entity = trigger.target();
    let mut entity_commands = commands
        .get_entity(entity)
        .expect("Triggered hotbar root");
    entity_commands.with_children(|builder| {
        let width = Val::Px(SLOT_SPRITE_SIZE);
        let height = Val::Px(SLOT_SPRITE_SIZE);
        let margin = UiRect::all(Val::Px(1.0));
        for i in 0..INVENTORY_WIDTH {
            builder.spawn((
                HotbarSlot,
                HotbarIndex(i),
                ImageNode::new(sprites.slot.clone()),
                Node {
                    width,
                    height,
                    margin,
                    ..default()
                },
            ));
        }
    });
}

fn update_selected_slot(
    selection: Query<&HotbarSelection, Changed<HotbarSelection>>,
    mut hotbar_display: Query<(&mut ImageNode, &HotbarIndex), With<HotbarSlot>>,
    sprites: Res<HotbarSprites>,
) {
    let Ok(HotbarSelection { index }) = selection.single() else {
        return;
    };
    for (mut image, hotbar_index) in hotbar_display.iter_mut() {
        image.image = if hotbar_index.0 as u8 == *index {
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
    let Ok(inventory) = q_inventory.single() else {
        return;
    };
    for (entity, HotbarIndex(index)) in q_hotbar_display.iter() {
        commands
            .entity(entity)
            .despawn_related::<Children>();
        let Some(item) = inventory.hotbar[*index] else {
            continue;
        };
        let material = match item.item {
            Item::Block(block) => block_icons
                .map
                .get(&block)
                .expect("Block should have a material for icon"),
        };
        let quantity_font_size = 18.0;
        let item_icon_id = commands
            .spawn((
                Ui,
                HotbarIndex(*index),
                ImageNode::new(material.clone_weak()),
                Node {
                    width: Val::Px(SLOT_SPRITE_SIZE),
                    height: Val::Px(SLOT_SPRITE_SIZE),
                    justify_content: JustifyContent::End,
                    align_content: AlignContent::FlexEnd,
                    ..default()
                },
            ))
            .with_children(|builder| {
                builder.spawn((
                    Ui,
                    Text::new(item.quantity),
                    TextFont {
                        font: font.0.clone_weak(),
                        font_size: quantity_font_size,
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Right),
                    Node {
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
                    },
                ));
            })
            .id();
        commands
            .entity(entity)
            .add_child(item_icon_id);
    }
}
