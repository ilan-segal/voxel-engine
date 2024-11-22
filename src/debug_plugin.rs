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
use perf_ui_targeted_block::PerfUiTargetedBlock;

mod chunk_border;
mod hitbox_frame;
mod perf_ui_camera_facing;
mod perf_ui_camera_pos;
mod perf_ui_targeted_block;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PerfUiPlugin,
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin,
            hitbox_frame::AabbWireframePlugin,
            chunk_border::ChunkBorderPlugin,
        ))
        .add_perf_ui_simple_entry::<PerfUiCameraPosition>()
        .add_perf_ui_simple_entry::<PerfUiCameraFacing>()
        .add_perf_ui_simple_entry::<PerfUiTargetedBlock>()
        .init_resource::<DebugUiIsVisible>()
        .add_systems(Startup, (setup, toggle_debug_ui).chain())
        .add_systems(
            Update,
            toggle_debug_ui.run_if(input_just_pressed(KeyCode::F3)),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PerfUiBundle::default(),
        PerfUiCameraPosition::default(),
        PerfUiCameraFacing::default(),
        PerfUiTargetedBlock::default(),
        DebugUi,
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
