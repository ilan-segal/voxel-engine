use bevy::prelude::*;

#[derive(Component, Default, Debug, PartialEq, Eq)]
pub enum FallingState {
    #[default]
    Falling,
    Grounded,
}
