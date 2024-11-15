use bevy::prelude::*;

const GRAVITY_ACCELERATION: f32 = 30.0;
const DEFAULT_GRAVITY: Vec3 = Vec3::new(0.0, -GRAVITY_ACCELERATION, 0.0);

#[derive(Component)]
pub struct Gravity(pub Vec3);

impl Default for Gravity {
    fn default() -> Self {
        Self(DEFAULT_GRAVITY)
    }
}
