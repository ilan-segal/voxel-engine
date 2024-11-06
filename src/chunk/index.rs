use super::{data::ChunkData, neighborhood::ChunkNeighborhood, position::ChunkPosition, Chunk};
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
}
