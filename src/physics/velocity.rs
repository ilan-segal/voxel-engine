use std::ops::{Add, Mul};

use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec3);

impl Add<Velocity> for Velocity {
    type Output = Self;

    fn add(self, rhs: Velocity) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Mul<f32> for Velocity {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}
