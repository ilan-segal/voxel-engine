use bevy::prelude::*;

use crate::{
    state::{AppState, InGameState},
    ui::{Ui, UiFont},
};
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FadeInTimer>()
            .add_systems(OnEnter(InGameState::Paused), spawn_menu)
            .add_systems(OnExit(InGameState::Paused), tear_down_menu)
            .add_systems(
                Update,
                (
                    update_timer,
                    fade_in_pause_menu_elements,
                    resume_game,
                    go_to_main_menu,
                    quit_game,
                )
                    .run_if(in_state(InGameState::Paused)),
            );
    }
}

#[derive(Resource, Default)]
struct FadeInTimer {
    seconds: f32,
}

fn update_timer(mut fade_in_timer: ResMut<FadeInTimer>, time: Res<Time>) {
    fade_in_timer.seconds += time.delta_secs();
}

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct PauseMenuButtonsRoot;

#[derive(Component)]
#[require(Button)]
struct ResumeButton;

#[derive(Component)]
#[require(Button)]
struct MainMenuButton;

#[derive(Component)]
#[require(Button)]
struct QuitGameButton;

const PAUSE_MENU_BUTTON_SPACING: Val = Val::Px(20.0);
const BUTTON_HEIGHT: Val = Val::Px(25.0);

fn spawn_menu(mut commands: Commands, ui_font: Res<UiFont>) {
    commands
        .spawn((
            Ui,
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|spawner| {
            spawner
                .spawn((
                    Ui,
                    PauseMenuButtonsRoot,
                    Node {
                        top: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        width: Val::Percent(30.0),
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: PAUSE_MENU_BUTTON_SPACING,
                        ..default()
                    },
                ))
                .with_children(|spawner| {
                    spawner.spawn((
                        Ui,
                        Node::default(),
                        Text::new("Paused"),
                        TextFont {
                            font_size: 30.0,
                            font: ui_font.0.clone_weak(),
                            ..default()
                        },
                    ));
                    spawner.spawn((
                        Ui,
                        ResumeButton,
                        Node {
                            height: BUTTON_HEIGHT,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        Text::new("Resume Game"),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font: ui_font.0.clone_weak(),
                            ..default()
                        },
                    ));
                    spawner.spawn((
                        Ui,
                        MainMenuButton,
                        Node {
                            height: BUTTON_HEIGHT,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        Text::new("Exit to Main Menu"),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font: ui_font.0.clone_weak(),
                            ..default()
                        },
                    ));
                    spawner.spawn((
                        Ui,
                        QuitGameButton,
                        Node {
                            height: BUTTON_HEIGHT,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        Text::new("Quit to Desktop"),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font: ui_font.0.clone_weak(),
                            ..default()
                        },
                    ));
                });
        });
    commands.insert_resource(FadeInTimer::default());
}

fn tear_down_menu(mut commands: Commands, q_root: Query<Entity, With<PauseMenuRoot>>) {
    for entity in q_root.iter() {
        commands.entity(entity).despawn();
    }
}

const FADE_IN_DURATION_SECONDS: f32 = 0.1;
const PAUSE_BACKGROUND_ALPHA: f32 = 0.75;

fn fade_in_pause_menu_elements(
    fade_in_timer: Res<FadeInTimer>,
    mut q_background: Query<&mut BackgroundColor, With<PauseMenuRoot>>,
) {
    let background_alpha =
        (fade_in_timer.seconds / FADE_IN_DURATION_SECONDS).min(1.0) * PAUSE_BACKGROUND_ALPHA;
    for mut background_color in q_background.iter_mut() {
        background_color.0 = Color::BLACK.with_alpha(background_alpha);
    }
}

fn resume_game(
    q_resume_button: Query<&Interaction, (With<ResumeButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    if let Ok(Interaction::Pressed) = q_resume_button.single() {
        next_state.set(InGameState::Playing);
    };
}

fn go_to_main_menu(
    q_button: Query<&Interaction, (With<MainMenuButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Ok(Interaction::Pressed) = q_button.single() {
        next_state.set(AppState::MainMenu);
    };
}

fn quit_game(
    q_button: Query<&Interaction, (With<QuitGameButton>, Changed<Interaction>)>,
    mut quit_events: EventWriter<AppExit>,
) {
    if let Ok(Interaction::Pressed) = q_button.single() {
        quit_events.write(AppExit::Success);
    };
}
