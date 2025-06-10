use bevy::prelude::*;

use crate::state::AppState;

use super::{UiFont, UiRoot};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_assets)
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnExit(AppState::MainMenu), tear_down_main_menu)
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
        .single()
        .expect("Menu root should exist");
    commands
        .entity(root)
        .with_children(|builder| {
            builder
                .spawn((
                    MainMenu,
                    Node {
                        width: HUNDRED_PERCENT,
                        height: HUNDRED_PERCENT,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|builder2| {
                    // Logo
                    builder2.spawn((
                        ImageNode::new(assets.logo.clone()),
                        Node {
                            width: LOGO_WIDTH * LOGO_SCALE,
                            height: LOGO_HEIGHT * LOGO_SCALE,
                            ..default()
                        },
                    ));
                    builder2
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: BUTTON_SPACING,
                            ..default()
                        })
                        .with_children(|buttons| {
                            // Play button
                            buttons
                                .spawn((
                                    PlayButton,
                                    Node {
                                        width: BUTTON_WIDTH,
                                        height: BUTTON_HEIGHT,
                                        ..Default::default()
                                    },
                                ))
                                .with_children(|text_builder| {
                                    text_builder.spawn((
                                        Text::new("Play"),
                                        TextFont {
                                            font: font.0.clone(),
                                            ..default()
                                        },
                                        TextLayout::new_with_justify(JustifyText::Center),
                                        Node {
                                            width: HUNDRED_PERCENT,
                                            ..default()
                                        },
                                    ));
                                });
                            // Quit button
                            buttons
                                .spawn((
                                    QuitButton,
                                    Node {
                                        width: BUTTON_WIDTH,
                                        height: BUTTON_HEIGHT,
                                        ..Default::default()
                                    },
                                ))
                                .with_children(|text_builder| {
                                    text_builder.spawn((
                                        Text::new("Quit"),
                                        TextFont {
                                            font: font.0.clone(),
                                            ..default()
                                        },
                                        TextLayout::new_with_justify(JustifyText::Center),
                                        Node {
                                            width: HUNDRED_PERCENT,
                                            ..default()
                                        },
                                    ));
                                });
                        });
                });
        });
}

fn tear_down_main_menu(q_root: Query<Entity, With<MainMenu>>, mut commands: Commands) {
    for entity in q_root.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
#[require(Button)]
struct PlayButton;

fn play_button(
    q_button: Query<&Interaction, (With<PlayButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in q_button.iter() {
        if let Interaction::Pressed = interaction {
            next_state.set(AppState::InGame);
        }
    }
}

#[derive(Component)]
#[require(Button)]
struct QuitButton;

fn quit_button(
    q_button: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
    mut quit_events: EventWriter<AppExit>,
) {
    for interaction in q_button.iter() {
        if let Interaction::Pressed = interaction {
            quit_events.write(AppExit::Success);
        }
    }
}
