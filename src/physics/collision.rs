use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Collidable;

#[derive(Event)]
pub struct Collision {
    pub entity: Entity,
    pub normal: Dir3,
}
