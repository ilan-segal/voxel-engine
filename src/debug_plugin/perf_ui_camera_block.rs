use bevy::{ecs::system::lifetimeless::SQuery, prelude::*};
use iyes_perf_ui::entry::PerfUiEntry;

use crate::{
    block::Block,
    player::{CameraBlock, Player},
};

#[derive(Component)]
pub struct PerfUiCameraBlock {
    sort_key: i32,
}

impl Default for PerfUiCameraBlock {
    fn default() -> Self {
        Self {
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

impl PerfUiEntry for PerfUiCameraBlock {
    type Value = Block;
    type SystemParam = SQuery<&'static CameraBlock, (With<Camera3d>, With<Player>)>;

    fn label(&self) -> &str {
        "Camera Block"
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        q_camera_pos: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        q_camera_pos.single().ok().map(|x| x.0)
        // .map(GlobalTransform::translation)
    }

    fn format_value(&self, value: &Self::Value) -> String {
        format!("{:?}", value)
    }
}
