use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
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
        .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PerfUiBundle::default(),
        PerfUiCameraPosition::default(),
        PerfUiCameraFacing::default(),
    ));
}
