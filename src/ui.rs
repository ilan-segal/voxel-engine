use bevy::prelude::*;

use crate::state::GameState;

pub mod block_icons;
mod crosshair;
mod health;
mod hotbar;
mod main_menu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crosshair::CrosshairPlugin,
            health::HealthUiPlugin,
            hotbar::HotbarUiPlugin,
            block_icons::BlockIconPlugin,
            main_menu::MainMenuPlugin,
        ))
        .add_systems(Startup, (spawn_ui_camera, (setup, create_ui_root)).chain())
        .add_systems(Update, update_button_colour)
        .add_systems(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::InGame,
            },
            despawn_ui_camera,
        );
    }
}

#[derive(Component)]
struct Ui;

#[derive(Component)]
struct UiRoot;

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFont(asset_server.load("ui/fonts/UbuntuMono-Regular.ttf")));
}

fn create_ui_root(mut commands: Commands) {
    commands.spawn((
        Ui,
        UiRoot,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
    ));
}

#[derive(Component)]
struct UiCamera;

fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera, UiCamera));
}

fn despawn_ui_camera(mut commands: Commands, q_camera: Query<Entity, With<UiCamera>>) {
    for entity in q_camera.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

const BUTTON_COLOUR: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_COLOUR_HOVER: Color = Color::srgb(0.5, 0.5, 0.5);
const BUTTON_COLOUR_PRESS: Color = Color::srgb(0.15, 0.15, 0.5);

fn update_button_colour(mut button: Query<(&mut BackgroundColor, &Interaction), With<Button>>) {
    for (mut colour, interaction) in button.iter_mut() {
        *colour = match interaction {
            Interaction::None => BUTTON_COLOUR,
            Interaction::Hovered => BUTTON_COLOUR_HOVER,
            Interaction::Pressed => BUTTON_COLOUR_PRESS,
        }
        .into();
    }
}
