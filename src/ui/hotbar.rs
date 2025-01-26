use bevy::prelude::*;

use crate::player::inventory::{HotbarSelection, HOTBAR_SIZE};

use super::Ui;

pub struct HotbarUiPlugin;

impl Plugin for HotbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.before(super::setup))
            .add_systems(Update, update_selected_slot)
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
    mut hotbar_display: Query<(&mut UiImage, &HotbarIndex)>,
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
