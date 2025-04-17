use bevy::prelude::*;

use crate::{
    block::Block, chunk::data::Blocks, utils::VolumetricRange, world::neighborhood::Neighborhood,
};

use super::{RandomUpdateEvent, SetBlockEvent};

pub struct DirtUpdatePlugin;

impl Plugin for DirtUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spread_grass);
    }
}

fn spread_grass(
    mut random_update_events: EventReader<RandomUpdateEvent>,
    mut set_block_events: EventWriter<SetBlockEvent>,
    q_blocks: Query<&Neighborhood<Blocks>>,
) {
    for update in random_update_events
        .read()
        .filter(|u| u.block == Block::Dirt)
    {
        let Ok(neighborhood) = q_blocks.get(update.chunk_id) else {
            continue;
        };
        let one_block_up = IVec3::new(0, 1, 0);
        let upper_pos = update.local_pos + one_block_up;
        let Some(upper_block) = neighborhood.at_pos(upper_pos) else {
            continue;
        };
        let covered = upper_block.is_solid();
        if covered {
            continue;
        }
        let grass_is_nearby = VolumetricRange::new(-1..2, -1..2, -1..2)
            .filter(|(x, y, z)| {
                let not_center = x != &0 || y != &0 || z != &0;
                let not_directly_below = x != &0 || z != &0 || y != &-1;
                not_center && not_directly_below
            })
            .any(|(x, y, z)| {
                let pos = update.local_pos + IVec3::new(x, y, z);
                neighborhood
                    .at_pos(pos)
                    .map(|block| block == &Block::Grass)
                    .unwrap_or(false)
            });
        if grass_is_nearby {
            let event = SetBlockEvent {
                block: Block::Grass,
                world_pos: update.world_pos.into(),
            };
            set_block_events.send(event);
        }
    }
}
