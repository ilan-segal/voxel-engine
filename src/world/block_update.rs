use bevy::prelude::*;

use crate::{
    block::Block,
    chunk::{data::Blocks, spatial::SpatiallyMapped, CHUNK_SIZE_I32},
};

use super::{index::ChunkIndex, WorldSet};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetBlockEvent>()
            .add_systems(Update, set_block.in_set(WorldSet));
    }
}

#[derive(Event, Debug)]
pub struct SetBlockEvent {
    pub block: Block,
    pub world_pos: [i32; 3],
}

fn set_block(
    chunk_index: Res<ChunkIndex>,
    mut block_events: EventReader<SetBlockEvent>,
    mut q_blocks: Query<&mut Blocks>,
) {
    for event in block_events.read() {
        let [x, y, z] = event.world_pos;
        let chunk_size = CHUNK_SIZE_I32;
        let chunk_x = x.div_floor(chunk_size);
        let chunk_y = y.div_floor(chunk_size);
        let chunk_z = z.div_floor(chunk_size);
        info!(
            "Setting block at {:?} to {:?}",
            event.world_pos, event.block
        );
        let local_x = (x - chunk_x * chunk_size) as usize;
        let local_y = (y - chunk_y * chunk_size) as usize;
        let local_z = (z - chunk_z * chunk_size) as usize;
        let Some(entity) = chunk_index
            .entity_by_pos
            .get(&IVec3::new(chunk_x, chunk_y, chunk_z))
        else {
            continue;
        };
        let Some(mut blocks) = q_blocks.get_mut(*entity).ok() else {
            continue;
        };
        *blocks.at_pos_mut([local_x, local_y, local_z]) = event.block;
    }
}
