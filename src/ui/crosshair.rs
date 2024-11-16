use bevy::prelude::*;

use super::{Ui, UiRoot};

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_crosshair.after(super::setup));
    }
}

const CROSSHAIR_WIDTH: f32 = 20.0;

fn spawn_crosshair(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_ui_root: Query<Entity, With<UiRoot>>,
) {
    let entity = q_ui_root
        .get_single()
        .ok()
        .expect("Should be exactly one UiRoot component");
    let crosshair_node = commands
        .spawn((
            Ui,
            NodeBundle {
                style: Style {
                    width: Val::Px(CROSSHAIR_WIDTH),
                    height: Val::Px(CROSSHAIR_WIDTH),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ..default()
            },
            UiImage::new(asset_server.load("crosshair.png")),
        ))
        .id();
    commands
        .entity(entity)
        .push_children(&[crosshair_node]);
}
