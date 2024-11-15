#![feature(let_chains)]
#![feature(float_next_up_down)]

use bevy::{
    input::mouse::MouseMotion,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::view::{Layer, RenderLayers},
    window::CursorGrabMode,
};
use physics::{aabb::Aabb, collision::Collidable, gravity::Gravity, velocity::Velocity};
use std::f32::consts::PI;

mod block;
mod camera_distance;
mod chunk;
mod debug_plugin;
mod mesh;
mod physics;
mod world;
mod world_noise;

const BLOCK_SIZE: f32 = 1.0;
const WORLD_LAYER: Layer = 0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voxel Engine".into(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            WireframePlugin,
            world::WorldPlugin,
            mesh::MeshPlugin,
            debug_plugin::DebugPlugin,
            chunk::ChunkPlugin,
            camera_distance::CameraDistancePlugin,
            physics::PhysicsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_camera, toggle_wireframe))
        .insert_resource(ClearColor(
            Color::linear_rgb(0.25, 0.60, 0.92).with_luminance(0.5),
        ))
        .insert_resource(Msaa::Sample8)
        .run();
}

fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    let camera_pos = Transform::from_xyz(0.0, 80.0, 0.0);

    commands.spawn((
        Camera3dBundle {
            transform: camera_pos.looking_to(Vec3::X, Vec3::Y),
            ..Default::default()
        },
        RenderLayers::layer(WORLD_LAYER),
        Aabb(Vec3::new(0.35, 2.0, 0.35)),
        Collidable,
        Gravity::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            color: Color::default(),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 200.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.) + Quat::from_rotation_z(-PI / 8.),
            ..default()
        },
        ..default()
    });
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Backquote) {
        wireframe_config.global = !wireframe_config.global;
    }
}

fn move_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut q_camera: Query<(&mut Velocity, &mut Transform), With<Camera3d>>,
) {
    const CAMERA_VERTICAL_BLOCKS_PER_SECOND: f32 = 10.0;
    const CAMERA_HORIZONTAL_BLOCKS_PER_SECOND: f32 = 10.0;
    for (mut v, mut transform) in q_camera.iter_mut() {
        v.0.x = 0.;
        v.0.z = 0.;
        if keys.pressed(KeyCode::Space) {
            v.0.y = CAMERA_VERTICAL_BLOCKS_PER_SECOND * BLOCK_SIZE;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            v.0.y -= CAMERA_VERTICAL_BLOCKS_PER_SECOND * BLOCK_SIZE;
        }
        let mut horizontal_movement = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            horizontal_movement.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            horizontal_movement.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            horizontal_movement.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            horizontal_movement.x += 1.0;
        }
        if horizontal_movement != Vec3::ZERO {
            let (yaw, _, _) = transform
                .rotation
                .to_euler(EulerRot::YXZ);
            let mut real_horizontal = (Quat::from_rotation_y(yaw) * horizontal_movement)
                .normalize()
                * CAMERA_HORIZONTAL_BLOCKS_PER_SECOND
                * BLOCK_SIZE;

            if keys.pressed(KeyCode::ControlLeft) {
                real_horizontal *= 10.0;
            }
            v.0 += real_horizontal;
        }

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
}
