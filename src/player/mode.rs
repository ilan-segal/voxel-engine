use bevy::prelude::*;

use super::Player;

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlayerMode {
    Survival,
    NoClip,
}

pub fn player_in_mode(
    mode: PlayerMode,
) -> impl FnMut(Query<&PlayerMode, With<Player>>) -> bool + Clone {
    move |q_player_mode: Query<&PlayerMode, With<Player>>| player_is_in_mode(q_player_mode, mode)
}

fn player_is_in_mode(q_player_mode: Query<&PlayerMode, With<Player>>, mode: PlayerMode) -> bool {
    let Ok(player_mode) = q_player_mode.single() else {
        return false;
    };
    return mode == *player_mode;
}
