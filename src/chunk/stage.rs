use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Noise,
    Terrain,
}
