use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum PlayerMode {
    Survival,
    NoClip,
}
