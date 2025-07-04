#![feature(let_chains)]
#![feature(int_roundings)]
#![feature(iter_map_windows)]
#![feature(step_trait)]

use bevy::{
    input::common_conditions::input_just_pressed,
    pbr::{
        light_consts::lux::CLEAR_SUNRISE,
        wireframe::{WireframeConfig, WireframePlugin},
    },
    prelude::*,
    window::CursorGrabMode,
};
use player::{Player, PlayerCamera};
use render_layer::WORLD_LAYER;
use state::AppState;

use crate::state::InGameState;

mod age;
mod block;
mod camera_distance;
mod chunk;
mod debug_plugin;
mod item;
mod physics;
mod player;
mod portal;
mod render;
mod render_layer;
mod state;
mod structure;
mod ui;
mod utils;
mod world;

const SKY_COLOUR: Color = Color::linear_rgb(0.25, 0.60, 0.92);
const TICKS_PER_SECOND: u8 = 20;

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
            WireframePlugin::default(),
            age::AgePlugin,
            camera_distance::CameraDistancePlugin,
            chunk::ChunkPlugin,
            debug_plugin::DebugPlugin,
            physics::PhysicsPlugin,
            player::PlayerPlugin,
            ui::UiPlugin,
            world::WorldPlugin,
            item::ItemPlugin,
            render::RenderPlugin,
            portal::PortalPlugin,
        ))
        .insert_state(AppState::Init)
        .add_sub_state::<InGameState>()
        .insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND as f64))
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(
            Update,
            (
                toggle_wireframe,
                go_to_main_menu.run_if(in_state(AppState::Init)),
                pause_game.run_if(
                    in_state(InGameState::Playing).and(input_just_pressed(KeyCode::Escape)),
                ),
                unpause_game
                    .run_if(in_state(InGameState::Paused).and(input_just_pressed(KeyCode::Escape))),
            ),
        )
        .add_systems(OnEnter(InGameState::Playing), lock_cursor)
        .add_systems(OnExit(InGameState::Playing), unlock_cursor)
        .insert_resource(ClearColor(SKY_COLOUR))
        .run();
}

fn setup_game(
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    mut gizmos_config_store: ResMut<GizmoConfigStore>,
) {
    let mut window = windows.single_mut().expect("Window component");
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;

    commands.spawn((
        Player,
        PlayerCamera,
        Transform::from_xyz(0.0, 2.0, 0.0).looking_to(Vec3::X, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: CLEAR_SUNRISE,
            shadows_enabled: false,
            ..default()
        },
        Transform::default().looking_to(Vec3::NEG_Y, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 750.,
        ..default()
    });

    for (_, config, _) in gizmos_config_store.iter_mut() {
        config.depth_bias = -0.001;
    }
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Backquote) {
        wireframe_config.global = !wireframe_config.global;
    }
}

fn go_to_main_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::MainMenu);
}

fn pause_game(mut next_state: ResMut<NextState<InGameState>>) {
    next_state.set(InGameState::Paused);
}

fn unpause_game(mut next_state: ResMut<NextState<InGameState>>) {
    next_state.set(InGameState::Playing);
}

fn lock_cursor(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut().expect("Window component");
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    let half_size = window.size() * 0.5;
    window.set_cursor_position(Some(half_size));
}

fn unlock_cursor(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut().expect("Window component");
    window.cursor_options.visible = true;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}
