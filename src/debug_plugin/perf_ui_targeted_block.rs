use bevy::{ecs::system::lifetimeless::SRes, prelude::*};
use iyes_perf_ui::entry::PerfUiEntry;

use crate::{
    block::Block, chunk::data::Blocks, player::block_target::TargetedBlock,
    world::neighborhood::ComponentIndex,
};

#[derive(Component)]
pub struct PerfUiTargetedBlock {
    sort_key: i32,
}

impl Default for PerfUiTargetedBlock {
    fn default() -> Self {
        Self {
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

impl PerfUiEntry for PerfUiTargetedBlock {
    type Value = Option<(IVec3, Block)>;
    type SystemParam = (SRes<TargetedBlock>, SRes<ComponentIndex<Blocks>>);

    fn label(&self) -> &str {
        "Targeted Block"
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        param: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        Some(param.0 .0.map(|pos| {
            (
                pos,
                param
                    .1
                    .at_pos(pos)
                    .cloned()
                    .unwrap_or_default(),
            )
        }))
    }

    fn format_value(&self, value: &Self::Value) -> String {
        match value {
            None => String::new(),
            Some((pos, block)) => {
                format!("{:}/{:}/{:} ({:?})", pos.x, pos.y, pos.z, block,)
            }
        }
    }
}
