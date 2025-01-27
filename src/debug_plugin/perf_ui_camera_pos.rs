use bevy::{ecs::system::lifetimeless::SQuery, prelude::*};
use iyes_perf_ui::entry::PerfUiEntry;

use crate::player::Player;

#[derive(Component)]
pub struct PerfUiCameraPosition {
    precision: usize,
    sort_key: i32,
}

impl Default for PerfUiCameraPosition {
    fn default() -> Self {
        Self {
            precision: 1,
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

impl PerfUiEntry for PerfUiCameraPosition {
    type Value = Vec3;
    type SystemParam = SQuery<&'static Transform, (With<Camera3d>, With<Player>)>;

    fn label(&self) -> &str {
        "Camera Position"
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        q_camera_pos: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        q_camera_pos
            .get_single()
            .ok()
            .map(|t| t.translation)
        // .map(GlobalTransform::translation)
    }

    fn format_value(&self, value: &Self::Value) -> String {
        format!(
            "X: {:.3$}, Y: {:.3$}, Z: {:.3$}",
            value.x, value.y, value.z, self.precision
        )
    }
}
