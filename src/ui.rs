use bevy::prelude::*;

use crate::state::AppState;

pub mod block_icons;
mod crosshair;
mod health;
mod hotbar;
mod main_menu;
mod pause_menu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crosshair::CrosshairPlugin,
            health::HealthUiPlugin,
            hotbar::HotbarUiPlugin,
            block_icons::BlockIconPlugin,
            main_menu::MainMenuPlugin,
            pause_menu::PauseMenuPlugin,
        ))
        .add_systems(Startup, (spawn_ui_camera, (setup, create_ui_root)).chain())
        .add_systems(Update, update_button_colour)
        .add_systems(
            OnTransition {
                exited: AppState::MainMenu,
                entered: AppState::InGame,
            },
            despawn_ui_camera,
        )
        .add_systems(
            OnTransition {
                exited: AppState::InGame,
                entered: AppState::MainMenu,
            },
            spawn_ui_camera,
        )
        .add_systems(OnExit(AppState::InGame), despawn_hud);
    }
}

#[derive(Component, Default)]
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
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

#[derive(Component)]
struct UiCamera;

fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera, UiCamera));
}

fn despawn_ui_camera(mut commands: Commands, q_camera: Query<Entity, With<UiCamera>>) {
    for entity in q_camera.iter() {
        commands.entity(entity).despawn();
    }
}

const BUTTON_COLOUR: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_COLOUR_HOVER: Color = Color::srgb(0.3, 0.3, 0.3);
const BUTTON_COLOUR_PRESS: Color = Color::srgb(0.5, 0.5, 0.5);

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

#[derive(Component, Default)]
#[require(Ui)]
struct HudUi;

fn despawn_hud(mut commands: Commands, q_hud: Query<Entity, With<HudUi>>) {
    for entity in q_hud.iter() {
        commands.entity(entity).try_despawn();
    }
}
