use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Stage {
    #[default]
    Noise,
    Terrain,
    Structures,
}
