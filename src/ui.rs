use bevy::prelude::*;
use health::HealthDisplayRoot;
use hotbar::HotbarDisplayRoot;

mod crosshair;
mod health;
mod hotbar;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crosshair::CrosshairPlugin,
            health::HealthUiPlugin,
            hotbar::HotbarUiPlugin,
        ))
        .add_systems(Startup, setup);
    }
}

#[derive(Component)]
struct Ui;

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct UiHotbar;

fn setup(mut commands: Commands) {
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
    commands
        .spawn((
            Ui,
            UiHotbar,
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::End,
                    justify_self: JustifySelf::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((Ui, HealthDisplayRoot, NodeBundle::default()));
            builder.spawn((
                Ui,
                HotbarDisplayRoot,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        align_content: AlignContent::Start,
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}
