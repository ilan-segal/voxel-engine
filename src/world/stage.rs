use bevy::prelude::*;

use crate::world::neighborhood::{CompleteNeighborhood, Neighborhood};

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Stage {
    #[default]
    Noise,
    Sculpt,
    Structures,
}

impl Stage {
    pub fn final_stage() -> Self {
        Self::Structures
    }
}
