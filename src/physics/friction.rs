use bevy::prelude::*;

use crate::physics::velocity::Velocity;

#[derive(Component, Default)]
#[require(Velocity)]
pub struct Friction {
    pub coefficient: f32,
}
