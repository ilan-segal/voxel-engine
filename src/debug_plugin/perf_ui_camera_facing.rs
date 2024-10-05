use bevy::{ecs::system::lifetimeless::SQuery, prelude::*};
use iyes_perf_ui::entry::PerfUiEntry;

use crate::block::BlockSide;

#[derive(Component)]
pub struct PerfUiCameraFacing {
    sort_key: i32,
}

impl Default for PerfUiCameraFacing {
    fn default() -> Self {
        Self {
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

impl PerfUiEntry for PerfUiCameraFacing {
    type Value = Dir3;
    type SystemParam = SQuery<&'static Transform, With<Camera3d>>;

    fn label(&self) -> &str {
        "Camera Facing"
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        transform: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        transform.get_single().ok().map(Transform::forward)
    }

    fn format_value(&self, value: &Self::Value) -> String {
        let v = value.as_vec3();
        format!(
            "{:?} ({:.2} / {:.2} / {:.2})",
            BlockSide::from(*value),
            v.x,
            v.y,
            v.z
        )
    }

    fn width_hint(&self) -> usize {
        29
    }
}
