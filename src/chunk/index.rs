use crate::block::Block;

use super::{
    data::ChunkData, neighborhood::ChunkNeighborhood, position::ChunkPosition, Chunk, CHUNK_SIZE,
};
use bevy::{prelude::*, utils::HashMap};
use itertools::Itertools;
use std::sync::Arc;

pub struct ChunkIndexPlugin;

impl Plugin for ChunkIndexPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkIndex>()
            .observe(on_chunk_loaded)
            .observe(on_chunk_unloaded);
    }
}

fn on_chunk_loaded(
    trigger: Trigger<OnAdd, Chunk>,
    query: Query<(Entity, &ChunkPosition, &Chunk)>,
    mut index: ResMut<ChunkIndex>,
) {
    let Ok((e, chunk_pos, chunk)) = query.get(trigger.entity()) else {
        return;
    };
    let data = chunk.blocks.clone();
    index.insert(chunk_pos.0, e, data);
}

fn on_chunk_unloaded(trigger: Trigger<OnRemove, Chunk>, mut index: ResMut<ChunkIndex>) {
    index.remove_entity(&trigger.entity());
}

#[derive(Resource, Default)]
pub struct ChunkIndex {
    chunk_map: HashMap<IVec3, Arc<ChunkData>>,
    pub entity_map: HashMap<IVec3, Entity>,
    pub pos_by_entity: HashMap<Entity, IVec3>,
}

impl ChunkIndex {
    pub fn get_neighborhood(&self, pos: &IVec3) -> ChunkNeighborhood {
        let mut chunks: [[[Option<Arc<ChunkData>>; 3]; 3]; 3] = Default::default();
        (-1..=1)
            .cartesian_product(-1..=1)
            .cartesian_product(-1..=1)
            .for_each(|((x, y), z)| {
                let cur_pos = IVec3::new(x, y, z) + *pos;
                chunks[(x + 1) as usize][(y + 1) as usize][(z + 1) as usize] =
                    self.chunk_map.get(&cur_pos).cloned();
            });
        return ChunkNeighborhood { chunks };
    }

    fn insert(&mut self, pos: IVec3, entity: Entity, data: Arc<ChunkData>) {
        self.chunk_map.insert(pos, data);
        self.entity_map.insert(pos, entity);
        self.pos_by_entity.insert(entity, pos);
    }

    fn remove_entity(&mut self, entity: &Entity) {
        if let Some(pos) = self.pos_by_entity.remove(entity) {
            self.chunk_map.remove(&pos);
            self.entity_map.remove(&pos);
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

        return chunk.at(local_x, local_y, local_z);
    }
}
