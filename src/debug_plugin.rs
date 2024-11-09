use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use iyes_perf_ui::{entries::PerfUiBundle, prelude::*};
use perf_ui_camera_facing::PerfUiCameraFacing;
use perf_ui_camera_pos::PerfUiCameraPosition;

mod perf_ui_camera_facing;
mod perf_ui_camera_pos;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PerfUiPlugin,
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin,
        ))
        .add_perf_ui_simple_entry::<PerfUiCameraPosition>()
        .add_perf_ui_simple_entry::<PerfUiCameraFacing>()
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_debug_ui, update_debug_ui_visibility))
        .init_resource::<DebugUiIsVisible>();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PerfUiBundle::default(),
        PerfUiCameraPosition::default(),
        PerfUiCameraFacing::default(),
        DebugUi,
    ));
}

#[derive(Component)]
struct DebugUi;

#[derive(Resource, Default)]
struct DebugUiIsVisible(bool);

fn update_debug_ui_visibility(
    mut query: Query<&mut Visibility, With<DebugUi>>,
    is_visible: Res<DebugUiIsVisible>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = if is_visible.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_debug_ui(
    mut key_events: EventReader<KeyboardInput>,
    mut is_visible: ResMut<DebugUiIsVisible>,
) {
    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            return;
        }
        match event.key_code {
            KeyCode::F1 => {
                is_visible.0 = !is_visible.0;
            }
            _ => {}
        }
    }
}
