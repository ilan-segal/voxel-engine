#![feature(let_chains)]
#![feature(float_next_up_down)]
#![feature(int_roundings)]
#![feature(iter_map_windows)]

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::view::RenderLayers,
    window::CursorGrabMode,
};
use player::PlayerBundle;
use render_layer::WORLD_LAYER;
use std::f32::consts::PI;

mod block;
mod camera_distance;
mod chunk;
mod controls;
mod cube_frame;
mod debug_plugin;
mod mesh;
mod physics;
mod player;
mod render_layer;
mod ui;
mod world;
mod world_noise;

const BLOCK_SIZE: f32 = 1.0;
const SKY_COLOUR: Color = Color::linear_rgb(0.25, 0.60, 0.92);

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
            camera_distance::CameraDistancePlugin,
            chunk::ChunkPlugin,
            controls::ControlsPlugin,
            cube_frame::FramePlugin,
            debug_plugin::DebugPlugin,
            mesh::MeshPlugin,
            physics::PhysicsPlugin,
            player::PlayerPlugin,
            ui::UiPlugin,
            world::WorldPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_wireframe)
        .insert_resource(ClearColor(SKY_COLOUR))
        .insert_resource(Msaa::Sample8)
        .run();
}

fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    commands.spawn((PlayerBundle::default(), RenderLayers::layer(WORLD_LAYER)));

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
