use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Friction {
    pub coefficient: f32,
}
