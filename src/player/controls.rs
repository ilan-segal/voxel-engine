use super::{falling_state::FallingState, IsJumping, IsSprinting, Player};
use crate::physics::{gravity::Gravity, velocity::Velocity};
use bevy::{ecs::query::QueryData, prelude::*};
use target_velocity::TargetVelocity;

mod keyboard_and_mouse;
pub mod target_velocity;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(keyboard_and_mouse::KeyboardMousePlugin)
            .add_systems(Update, update_velocity);
    }
}

/// m/s^2
const GROUND_ACCELERATION: f32 = 100.0;
/// m/s
const WALK_SPEED: f32 = 4.5;
/// m/s
const RUN_SPEED: f32 = WALK_SPEED * 1.5;
/// m
const JUMP_HEIGHT: f32 = 1.25;

#[derive(QueryData)]
#[query_data(mutable)]
struct PlayerVelocityData {
    velocity: &'static mut Velocity,
    target_velocity: &'static TargetVelocity,
    gravity: &'static Gravity,
    falling_state: &'static FallingState,
    is_sprinting: &'static IsSprinting,
    is_jumping: &'static IsJumping,
}

fn update_velocity(mut q: Query<PlayerVelocityData, With<Player>>, time: Res<Time>) {
    for mut player in q.iter_mut() {
        // Jumping
        if player.is_jumping.0 && player.falling_state == &FallingState::Grounded {
            let jump_speed = square_root_v(-2.0 * player.gravity.0 * JUMP_HEIGHT);
            player.velocity.0 += jump_speed;
        }
        // Walking
        let v_target = player.target_velocity.0.with_y(0.0)
            * if player.is_sprinting.0 {
                RUN_SPEED
            } else {
                WALK_SPEED
            };
        let v_actual = player.velocity.0.with_y(0.0);
        let diff = v_target - v_actual;
        let dv_length = GROUND_ACCELERATION * time.delta_seconds();
        if diff.length() <= dv_length {
            player.velocity.0.x = v_target.x;
            player.velocity.0.z = v_target.z;
        } else {
            let dv = dv_length * diff.normalize();
            player.velocity.0 += dv;
        }
    }
}

/// Elementwise sqrt(abs(v)) preserving sign
fn square_root_v(v: Vec3) -> Vec3 {
    let [x, y, z] = v.abs().to_array();
    Vec3::new(x.sqrt(), y.sqrt(), z.sqrt()) * v.signum()
}
