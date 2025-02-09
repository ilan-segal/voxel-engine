use bevy::prelude::*;

use crate::state::GameState;

use super::{UiFont, UiRoot};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_assets)
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), tear_down_main_menu)
            .add_systems(Update, (play_button, quit_button));
    }
}

#[derive(Resource)]
struct MainMenuAssets {
    logo: Handle<Image>,
}

fn setup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let logo = asset_server.load("menu/logo.png");
    commands.insert_resource(MainMenuAssets { logo });
}

#[derive(Component)]
struct MainMenu;

const HUNDRED_PERCENT: Val = Val::Percent(100.0);
const LOGO_SCALE: f32 = 2.0;
const LOGO_WIDTH: Val = Val::Px(150.0);
const LOGO_HEIGHT: Val = Val::Px(100.0);
const BUTTON_WIDTH: Val = Val::Px(150.0);
const BUTTON_HEIGHT: Val = Val::Px(25.0);
const BUTTON_SPACING: Val = Val::Px(25.0);

fn setup_main_menu(
    q_root: Query<Entity, With<UiRoot>>,
    mut commands: Commands,
    assets: Res<MainMenuAssets>,
    font: Res<UiFont>,
) {
    let root = q_root
        .get_single()
        .expect("Menu root should exist");
    commands
        .entity(root)
        .with_children(|builder| {
            builder
                .spawn((
                    MainMenu,
                    NodeBundle {
                        style: Style {
                            width: HUNDRED_PERCENT,
                            height: HUNDRED_PERCENT,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|builder2| {
                    // Logo
                    builder2.spawn((
                        NodeBundle {
                            style: Style {
                                width: LOGO_WIDTH * LOGO_SCALE,
                                height: LOGO_HEIGHT * LOGO_SCALE,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        UiImage::new(assets.logo.clone()),
                    ));
                    builder2
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                row_gap: BUTTON_SPACING,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|buttons| {
                            // Play button
                            buttons
                                .spawn((
                                    PlayButton,
                                    ButtonBundle {
                                        style: Style {
                                            width: BUTTON_WIDTH,
                                            height: BUTTON_HEIGHT,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                ))
                                .with_children(|text_builder| {
                                    text_builder.spawn(
                                        TextBundle::from_section(
                                            "Play",
                                            TextStyle {
                                                font: font.0.clone(),
                                                ..default()
                                            },
                                        )
                                        .with_text_justify(JustifyText::Center)
                                        .with_style(Style {
                                            width: HUNDRED_PERCENT,
                                            ..default()
                                        }),
                                    );
                                });
                            // Quit button
                            buttons
                                .spawn((
                                    QuitButton,
                                    ButtonBundle {
                                        style: Style {
                                            width: BUTTON_WIDTH,
                                            height: BUTTON_HEIGHT,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                ))
                                .with_children(|text_builder| {
                                    text_builder.spawn(
                                        TextBundle::from_section(
                                            "Quit",
                                            TextStyle {
                                                font: font.0.clone(),
                                                ..default()
                                            },
                                        )
                                        .with_text_justify(JustifyText::Center)
                                        .with_style(Style {
                                            width: HUNDRED_PERCENT,
                                            ..default()
                                        }),
                                    );
                                });
                        });
                });
        });
}

fn tear_down_main_menu(q_root: Query<Entity, With<MainMenu>>, mut commands: Commands) {
    for entity in q_root.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

#[derive(Component)]
struct PlayButton;

fn play_button(
    q_button: Query<&Interaction, (With<PlayButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in q_button.iter() {
        if let Interaction::Pressed = interaction {
            next_state.set(GameState::InGame);
        }
    }
}

#[derive(Component)]
struct QuitButton;

fn quit_button(
    q_button: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
    mut quit_events: EventWriter<AppExit>,
) {
    for interaction in q_button.iter() {
        if let Interaction::Pressed = interaction {
            quit_events.send(AppExit::Success);
        }
    }
}
