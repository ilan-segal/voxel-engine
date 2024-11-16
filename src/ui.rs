use bevy::prelude::*;

mod crosshair;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crosshair::CrosshairPlugin)
            .add_systems(Startup, setup);
    }
}

#[derive(Component)]
struct Ui;

#[derive(Component)]
struct UiRoot;

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
}
