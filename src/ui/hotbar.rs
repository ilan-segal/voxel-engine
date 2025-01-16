use bevy::prelude::*;

use super::Ui;

pub struct HotbarUiPlugin;

impl Plugin for HotbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.before(super::setup))
            .observe(add_slots);
    }
}

#[derive(Component)]
pub struct HotbarDisplayRoot;

#[derive(Resource)]
struct HotbarSprites {
    slot: UiImage,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprites = HotbarSprites {
        slot: UiImage::new(asset_server.load("ui/hud/inventory/slot.png")),
    };
    commands.insert_resource(sprites);
}

const SLOT_SPRITE_SIZE: f32 = 50.0;
const HOTBAR_SIZE: u8 = 10;

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
        for _ in 0..HOTBAR_SIZE {
            builder.spawn((
                Ui,
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
