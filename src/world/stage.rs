use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Stage {
    #[default]
    Noise,
    Terrain,
    Structures,
}

impl Stage {
    pub fn final_stage() -> Self {
        Self::Structures
    }
}
