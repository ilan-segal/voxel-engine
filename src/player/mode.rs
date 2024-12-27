use bevy::prelude::*;

#[derive(Component, Debug)]
pub enum PlayerMode {
    Survival,
    NoClip,
}
