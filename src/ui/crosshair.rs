use bevy::prelude::*;

use crate::{state::AppState, ui::HudUi};

use super::UiRoot;

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_crosshair);
    }
}

const CROSSHAIR_WIDTH: f32 = 20.0;

fn spawn_crosshair(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_ui_root: Query<Entity, With<UiRoot>>,
) {
    let entity = q_ui_root
        .single()
        .ok()
        .expect("Should be exactly one UiRoot component");
    let crosshair_node = commands
        .spawn((
            HudUi,
            Node {
                width: Val::Px(CROSSHAIR_WIDTH),
                height: Val::Px(CROSSHAIR_WIDTH),
                align_self: AlignSelf::Center,
                ..default()
            },
            ImageNode::new(asset_server.load("ui/crosshair.png")),
        ))
        .id();
    commands
        .entity(entity)
        .add_child(crosshair_node);
}
