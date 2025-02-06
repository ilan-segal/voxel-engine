#![feature(let_chains)]
#![feature(float_next_up_down)]
#![feature(int_roundings)]
#![feature(iter_map_windows)]
#![feature(trivial_bounds)]
#![feature(step_trait)]

use bevy::{
    core_pipeline::smaa::SmaaSettings,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::view::GpuCulling,
    window::CursorGrabMode,
};
use player::PlayerBundle;
use render_layer::WORLD_LAYER;
use state::GameState;

mod age;
mod block;
mod camera_distance;
mod chunk;
mod cube_frame;
mod debug_plugin;
mod item;
mod mesh;
mod physics;
mod player;
mod render_layer;
mod state;
mod structure;
mod texture;
mod ui;
mod utils;
mod world;

const BLOCK_SIZE: f32 = 1.0;
const SKY_COLOUR: Color = Color::linear_rgb(0.25, 0.60, 0.92);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Engine".into(),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            WireframePlugin,
            age::AgePlugin,
            camera_distance::CameraDistancePlugin,
            chunk::ChunkPlugin,
            cube_frame::FramePlugin,
            debug_plugin::DebugPlugin,
            // file_io::FileIOPlugin,
            mesh::MeshPlugin,
            physics::PhysicsPlugin,
            player::PlayerPlugin,
            texture::TexturePlugin,
            ui::UiPlugin,
            world::WorldPlugin,
            item::ItemPlugin,
        ))
        .insert_state(GameState::InGame)
        .add_systems(OnEnter(GameState::InGame), setup_game)
        .add_systems(Update, toggle_wireframe)
        .insert_resource(ClearColor(SKY_COLOUR))
        .insert_resource(Msaa::Sample8)
        .run();
}

fn setup_game(mut commands: Commands, mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    commands.spawn((PlayerBundle::default(), SmaaSettings::default(), GpuCulling));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 750.,
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
