use bevy::prelude::*;

use crate::{block::Block, chunk::data::Blocks, world::neighborhood::Neighborhood};

use super::{RandomUpdateEvent, SetBlockEvent};

pub struct GrassUpdatePlugin;

impl Plugin for GrassUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, smother_grass);
    }
}

fn smother_grass(
    mut random_update_events: EventReader<RandomUpdateEvent>,
    mut set_block_events: EventWriter<SetBlockEvent>,
    q_blocks: Query<&Neighborhood<Blocks>>,
) {
    for update in random_update_events
        .read()
        .filter(|u| u.block == Block::Grass)
    {
        let Ok(neighborhood) = q_blocks.get(update.chunk_id) else {
            continue;
        };
        let one_block_up = IVec3::new(0, 1, 0);
        let upper_pos = update.local_pos + one_block_up;
        let Some(upper_block) = neighborhood.at_pos(upper_pos) else {
            continue;
        };
        if upper_block.is_solid() {
            // The grass on this block gets smothered
            let event = SetBlockEvent {
                block: Block::Dirt,
                world_pos: update.world_pos.into(),
            };
            set_block_events.write(event);
        }
    }
}
