use crate::{
    block::Block,
    chunk::{
        data::Blocks, position::ChunkPosition, spatial::SpatiallyMapped, CHUNK_SIZE, CHUNK_SIZE_I32,
    },
    world::{index::ChunkIndex, neighborhood::Neighborhood, stage::Stage, WorldSet},
};
use bevy::{ecs::query::QueryData, prelude::*};

mod dirt;
mod grass;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetBlockEvent>()
            .add_event::<RandomUpdateEvent>()
            .init_resource::<RandomTickSpeed>()
            .add_systems(Update, set_block.in_set(WorldSet))
            .add_systems(FixedUpdate, do_random_block_updates)
            .add_plugins((dirt::DirtUpdatePlugin, grass::GrassUpdatePlugin));
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

#[derive(Resource)]
pub struct RandomTickSpeed(pub u8);

impl Default for RandomTickSpeed {
    fn default() -> Self {
        Self(24)
    }
}

#[derive(Event)]
pub struct RandomUpdateEvent {
    pub world_pos: IVec3,
    /// Position in the chunk
    pub local_pos: IVec3,
    pub chunk_id: Entity,
    pub block: Block,
}

#[derive(QueryData)]
struct RandomBlockUpdateQueryData {
    chunk_id: Entity,
    stage_neighborhood: &'static Neighborhood<Stage>,
    block_neighborhood: &'static Neighborhood<Blocks>,
    chunk_position: &'static ChunkPosition,
}

fn do_random_block_updates(
    q_chunk: Query<RandomBlockUpdateQueryData>,
    random_tick_speed: Res<RandomTickSpeed>,
    mut update_events: EventWriter<RandomUpdateEvent>,
) {
    let ticks = random_tick_speed.0;
    for chunk in q_chunk.iter() {
        if chunk
            .stage_neighborhood
            .min()
            .unwrap()
            .as_ref()
            != &Stage::final_stage()
        {
            continue;
        }
        for _ in 0..ticks {
            let local_pos = get_random_update_location();
            let Some(block) = chunk
                .block_neighborhood
                .at_pos(local_pos)
                .cloned()
            else {
                continue;
            };
            let chunk_id = chunk.chunk_id;
            let world_pos = local_pos + chunk.chunk_position.0 * CHUNK_SIZE_I32;
            let update = RandomUpdateEvent {
                world_pos,
                local_pos,
                chunk_id,
                block,
            };
            update_events.write(update);
        }
    }
}

fn get_random_update_location() -> IVec3 {
    [
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
    ]
    .map(|x| x % CHUNK_SIZE as u8)
    .map(|x| x as i32)
    .into()
}
