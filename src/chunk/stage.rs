use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Stage {
    #[default]
    Noise,
    Terrain,
    Structures,
}

impl Stage {
    pub fn last() -> Self {
        Self::Structures
    }
}
