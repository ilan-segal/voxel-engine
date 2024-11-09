use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use iyes_perf_ui::{entries::PerfUiBundle, prelude::*};
use perf_ui_camera_facing::PerfUiCameraFacing;
use perf_ui_camera_pos::PerfUiCameraPosition;

mod hitbox_frame;
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
            hitbox_frame::AabbWireframePlugin,
        ))
        .add_perf_ui_simple_entry::<PerfUiCameraPosition>()
        .add_perf_ui_simple_entry::<PerfUiCameraFacing>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            toggle_debug_ui.run_if(input_just_pressed(KeyCode::F3)),
        )
        .init_resource::<DebugUiIsVisible>();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PerfUiBundle::default(),
        PerfUiCameraPosition::default(),
        PerfUiCameraFacing::default(),
        DebugUi,
        Visibility::Hidden,
    ));
}

#[derive(Component)]
struct DebugUi;

#[derive(Resource, Default)]
struct DebugUiIsVisible(bool);

fn toggle_debug_ui(
    mut query: Query<&mut Visibility, With<DebugUi>>,
    mut is_visible: ResMut<DebugUiIsVisible>,
) {
    is_visible.0 = !is_visible.0;
    for mut visibility in query.iter_mut() {
        *visibility = if is_visible.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
