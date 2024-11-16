use bevy::prelude::*;
use block_target::BlockTargetPlugin;

pub mod block_target;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlockTargetPlugin);
    }
}

#[derive(Component)]
pub struct Player;
