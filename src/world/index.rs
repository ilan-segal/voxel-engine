use crate::block::Block;
use crate::chunk::Chunk;
use crate::chunk::{data::Blocks, position::ChunkPosition, CHUNK_SIZE};
use bevy::{prelude::*, utils::HashMap};
use itertools::Itertools;
use std::sync::Arc;

use super::neighborhood::BlockNeighborhood;
use super::neighborhood::NeighborhoodComponentCopy;

pub struct ChunkIndexPlugin;

impl Plugin for ChunkIndexPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkIndex>()
            .observe(on_blocks_changed)
            .observe(on_blocks_loaded)
            .observe(on_chunk_unloaded);
    }
}

pub fn on_blocks_changed(
    trigger: Trigger<OnAdd, NeighborhoodComponentCopy<Blocks>>,
    query: Query<(&ChunkPosition, &NeighborhoodComponentCopy<Blocks>)>,
    mut index: ResMut<ChunkIndex>,
) {
    let e = trigger.entity();
    let Ok((pos, blocks)) = query.get(e) else {
        return;
    };
    index.insert(pos.0, e, blocks.0.clone());
}

pub fn on_blocks_loaded(
    trigger: Trigger<OnAdd, ChunkPosition>,
    query: Query<&ChunkPosition, With<Chunk>>,
    mut index: ResMut<ChunkIndex>,
) {
    let e = trigger.entity();
    let Ok(pos) = query.get(e) else {
        return;
    };
    index.insert_entity(pos.0, e);
}

fn on_chunk_unloaded(trigger: Trigger<OnRemove, Chunk>, mut index: ResMut<ChunkIndex>) {
    index.remove_entity(&trigger.entity());
}

#[derive(Resource, Default)]
pub struct ChunkIndex {
    chunk_map: HashMap<IVec3, Arc<Blocks>>,
    pub entity_by_pos: HashMap<IVec3, Entity>,
    pub pos_by_entity: HashMap<Entity, IVec3>,
}

impl ChunkIndex {
    fn get_neighborhood(&self, pos: &IVec3) -> BlockNeighborhood {
        let mut chunks: [[[Option<Arc<Blocks>>; 3]; 3]; 3] = Default::default();
        (-1..=1)
            .cartesian_product(-1..=1)
            .cartesian_product(-1..=1)
            .for_each(|((x, y), z)| {
                let cur_pos = IVec3::new(x, y, z) + *pos;
                chunks[(x + 1) as usize][(y + 1) as usize][(z + 1) as usize] =
                    self.chunk_map.get(&cur_pos).cloned();
            });
        return BlockNeighborhood { chunks };
    }

    fn insert(&mut self, pos: IVec3, entity: Entity, blocks: Arc<Blocks>) {
        self.chunk_map.insert(pos, blocks);
        self.entity_by_pos.insert(pos, entity);
        self.pos_by_entity.insert(entity, pos);
    }

    fn insert_entity(&mut self, pos: IVec3, entity: Entity) {
        self.entity_by_pos.insert(pos, entity);
        self.pos_by_entity.insert(entity, pos);
    }

    fn remove_entity(&mut self, entity: &Entity) {
        if let Some(pos) = self.pos_by_entity.remove(entity) {
            self.chunk_map.remove(&pos);
            self.entity_by_pos.remove(&pos);
        }
    }

    pub fn at_pos(&self, pos: &IVec3) -> Block {
        self.at_i(pos.x, pos.y, pos.z)
    }

    pub fn at(&self, x: f32, y: f32, z: f32) -> Block {
        self.at_i(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    pub fn at_i(&self, x: i32, y: i32, z: i32) -> Block {
        let chunk_size = CHUNK_SIZE as i32;

        let chunk_x = x.div_floor(chunk_size);
        let chunk_y = y.div_floor(chunk_size);
        let chunk_z = z.div_floor(chunk_size);

        let chunk = self.get_neighborhood(&IVec3::new(chunk_x, chunk_y, chunk_z));

        let local_x = x - chunk_x * chunk_size;
        let local_y = y - chunk_y * chunk_size;
        let local_z = z - chunk_z * chunk_size;

        return chunk
            .block_at(local_x, local_y, local_z)
            .copied()
            .unwrap_or_default();
    }
}
