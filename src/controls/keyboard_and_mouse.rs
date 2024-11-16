use std::f32::consts::PI;

use bevy::{ecs::query::QueryData, input::mouse::MouseMotion, prelude::*};

use crate::{
    physics::{gravity::Gravity, velocity::Velocity, PhysicsSystemSet},
    player::{falling_state::FallingState, Player},
};

pub struct KeyboardMousePlugin;

impl Plugin for KeyboardMousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                rotate_camera_with_mouse,
                process_keyboard_inputs.before(PhysicsSystemSet::Act),
            ),
        );
    }
}

fn rotate_camera_with_mouse(
    mut mouse_events: EventReader<MouseMotion>,
    mut q_camera: Query<&mut Transform, (With<Player>, With<Camera3d>)>,
) {
    let mut transform = q_camera
        .get_single_mut()
        .ok()
        .expect("There should be exactly one player camera");
    const CAMERA_MOUSE_SENSITIVITY_X: f32 = 0.004;
    const CAMERA_MOUSE_SENSITIVITY_Y: f32 = 0.0025;
    for MouseMotion { delta } in mouse_events.read() {
        transform.rotate_axis(Dir3::NEG_Y, delta.x * CAMERA_MOUSE_SENSITIVITY_X);
        let (yaw, mut pitch, _) = transform
            .rotation
            .to_euler(EulerRot::YXZ);
        pitch = (pitch - delta.y * CAMERA_MOUSE_SENSITIVITY_Y).clamp(-PI * 0.5, PI * 0.5);
        transform.rotation = Quat::from_euler(
            // YXZ order corresponds to the common
            // "yaw"/"pitch"/"roll" convention
            EulerRot::YXZ,
            yaw,
            pitch,
            0.0,
        );
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct KeyboardInputQuery {
    v: &'static mut Velocity,
    t: &'static Transform,
    g: &'static Gravity,
    falling_state: &'static FallingState,
}

fn process_keyboard_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_velocity: Query<KeyboardInputQuery, With<Player>>,
) {
    const WALK_SPEED: f32 = 4.5;
    const RUN_SPEED: f32 = WALK_SPEED * 1.5;
    let Ok(mut object) = q_velocity.get_single_mut() else {
        return;
    };
    object.v.0.x = 0.;
    object.v.0.z = 0.;
    let mut v_horizontal = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        v_horizontal.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        v_horizontal.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        v_horizontal.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        v_horizontal.x += 1.0;
    }
    if v_horizontal != Vec3::ZERO {
        let (yaw, _, _) = object
            .t
            .rotation
            .to_euler(EulerRot::YXZ);
        v_horizontal = (Quat::from_rotation_y(yaw) * v_horizontal).normalize();
        if keys.pressed(KeyCode::ControlLeft) {
            v_horizontal *= RUN_SPEED;
        } else {
            v_horizontal *= WALK_SPEED;
        }
        object.v.0 += v_horizontal;
    }

    const JUMP_HEIGHT: f32 = 1.25;
    let jump_speed = square_root_v(-2.0 * object.g.0 * JUMP_HEIGHT);
    if keys.pressed(KeyCode::Space) && object.falling_state == &FallingState::Grounded {
        object.v.0 = jump_speed;
    }
}

/// Elementwise sqrt(abs(v)) preserving sign
fn square_root_v(v: Vec3) -> Vec3 {
    let [x, y, z] = v.abs().to_array();
    Vec3::new(x.sqrt(), y.sqrt(), z.sqrt()) * v.signum()
}
