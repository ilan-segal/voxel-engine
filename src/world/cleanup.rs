use bevy::prelude::*;

use crate::{chunk::Chunk, item::DroppedItem, player::Player, state::AppState};

pub struct CleanupPlugin;

impl Plugin for CleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::InGame), despawn);
    }
}

fn despawn(
    mut commands: Commands,
    q_despawnable: Query<
        Entity,
        Or<(
            With<Chunk>,
            With<DroppedItem>,
            With<Player>,
            With<DirectionalLight>,
        )>,
    >,
) {
    for entity in q_despawnable.iter() {
        commands.entity(entity).try_despawn();
    }
}
