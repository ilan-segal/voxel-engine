use crate::chunk::{
    data::{Blocks, Noise3d, Perlin2d},
    position::ChunkPosition,
    CHUNK_SIZE,
};
use crate::{block::Block, mesh::MeshSet};
use bevy::{ecs::query::QueryData, prelude::*, utils::HashMap};
use itertools::Itertools;
use std::sync::Arc;

use super::{chunk_neighborhood::ChunkNeighborhood, stage::Stage, ChunkBundle, WorldSet};

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
            .observe(on_chunk_unloaded);
    }
}

#[derive(QueryData)]
pub struct ChunkBundleQueryData {
    chunk_pos: &'static ChunkPosition,
    stage: &'static Stage,
    blocks: &'static Blocks,
    perlin_2d: &'static Perlin2d,
    noise_3d: &'static Noise3d,
}

pub fn on_blocks_changed(
    query: Query<(Entity, ChunkBundleQueryData), Changed<Blocks>>,
    mut index: ResMut<ChunkIndex>,
) {
    for (e, bundle) in query.iter() {
        index.insert(
            bundle.chunk_pos.0,
            e,
            bundle.blocks,
            bundle.stage,
            bundle.perlin_2d,
            bundle.noise_3d,
        );
    }
}

fn on_chunk_unloaded(trigger: Trigger<OnRemove, Blocks>, mut index: ResMut<ChunkIndex>) {
    index.remove_entity(&trigger.entity());
}

#[derive(Resource, Default)]
pub struct ChunkIndex {
    chunk_map: HashMap<IVec3, Arc<ChunkBundle>>,
    pub entity_by_pos: HashMap<IVec3, Entity>,
    pub pos_by_entity: HashMap<Entity, IVec3>,
}

impl ChunkIndex {
    pub fn get_neighborhood(&self, pos: &IVec3) -> ChunkNeighborhood {
        let mut chunks: [[[Option<Arc<ChunkBundle>>; 3]; 3]; 3] = Default::default();
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

    fn insert(
        &mut self,
        pos: IVec3,
        entity: Entity,
        blocks: &Blocks,
        stage: &Stage,
        perlin_2d: &Perlin2d,
        noise_3d: &Noise3d,
    ) {
        let data = Arc::new(ChunkBundle {
            blocks: (*blocks).clone(),
            stage: *stage,
            perlin_2d: (*perlin_2d).clone(),
            noise_3d: (*noise_3d).clone(),
        });
        self.chunk_map.insert(pos, data);
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
