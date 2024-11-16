use bevy::prelude::*;

#[derive(Component)]
pub struct Collidable;

#[derive(Event)]
pub struct Collision {
    pub entity: Entity,
    pub normal: Dir3,
}
