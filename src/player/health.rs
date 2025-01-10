use bevy::prelude::*;

#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component)]
pub struct MaxHealth(pub u32);
