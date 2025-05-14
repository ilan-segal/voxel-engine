use crate::chunk::{data::Blocks, position::ChunkPosition, Chunk};
use crate::mesh::MeshSet;
use bevy::ecs::entity::EntityHashMap;
use bevy::{platform::collections::HashMap, prelude::*};

use super::WorldSet;

pub struct ChunkIndexPlugin;

impl Plugin for ChunkIndexPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkIndex>()
            .add_systems(
                Update,
                on_blocks_changed
                    .before(MeshSet)
                    .before(WorldSet),
            )
            .add_observer(on_chunk_loaded)
            .add_observer(on_chunk_unloaded);
    }
}

pub fn on_blocks_changed(
    query: Query<(Entity, &ChunkPosition), Changed<Blocks>>,
    mut index: ResMut<ChunkIndex>,
) {
    for (e, position) in query.iter() {
        index.insert(position.0, e);
    }
}

fn on_chunk_unloaded(trigger: Trigger<OnRemove, Blocks>, mut index: ResMut<ChunkIndex>) {
    index.remove_entity(&trigger.target());
}

fn on_chunk_loaded(
    trigger: Trigger<OnAdd, Chunk>,
    q_pos: Query<&ChunkPosition>,
    mut index: ResMut<ChunkIndex>,
) {
    let entity = trigger.target();
    let Ok(pos) = q_pos.get(entity) else {
        return;
    };
    index
        .entity_by_pos
        .insert(pos.0, entity);
    index
        .pos_by_entity
        .insert(entity, pos.0);
}

#[derive(Resource, Default)]
pub struct ChunkIndex {
    pub entity_by_pos: HashMap<IVec3, Entity>,
    pub pos_by_entity: EntityHashMap<IVec3>,
}

impl ChunkIndex {
    fn insert(&mut self, pos: IVec3, entity: Entity) {
        self.entity_by_pos.insert(pos, entity);
        self.pos_by_entity.insert(entity, pos);
    }

    fn remove_entity(&mut self, entity: &Entity) {
        if let Some(pos) = self.pos_by_entity.remove(entity) {
            self.entity_by_pos.remove(&pos);
        }
    }
}
