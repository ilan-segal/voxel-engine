use std::sync::Arc;

use crate::{
    block::{Block, BlockSide},
    chunk_data::{ChunkData, CHUNK_SIZE},
};
use bevy::{prelude::*, utils::HashMap};
use itertools::Itertools;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.observe(on_chunk_loaded)
            .observe(on_chunk_unloaded)
            .init_resource::<ChunkIndex>();
    }
}

// 32x32x32 chunk of blocks
#[derive(Component, Clone)]
pub struct Chunk {
    // x, y, z
    pub blocks: Arc<ChunkData>,
}

impl Chunk {
    pub fn new(data: ChunkData) -> Self {
        Self {
            blocks: Arc::new(data),
        }
    }
}

#[derive(Component, PartialEq, Eq, Default, Hash, Clone, Copy)]
pub struct ChunkPosition(pub IVec3);

impl ChunkPosition {
    pub fn from_world_position(p: &Vec3) -> Self {
        ChunkPosition(
            (*p / (CHUNK_SIZE as f32))
                .floor()
                .as_ivec3(),
        )
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
    index
        .chunk_map
        .insert(chunk_pos.0, data);
    index.entity_map.insert(chunk_pos.0, e);
}

fn on_chunk_unloaded(
    trigger: Trigger<OnRemove, Chunk>,
    query: Query<&ChunkPosition, With<Chunk>>,
    mut index: ResMut<ChunkIndex>,
) {
    let Ok(chunk_pos) = query.get(trigger.entity()) else {
        return;
    };
    index.chunk_map.remove(&chunk_pos.0);
}

#[derive(Resource, Default)]
pub struct ChunkIndex {
    pub chunk_map: HashMap<IVec3, Arc<ChunkData>>,
    pub entity_map: HashMap<IVec3, Entity>,
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
}

/*
Represents a 3x3x3 cube of chunks
*/
pub struct ChunkNeighborhood {
    chunks: [[[Option<Arc<ChunkData>>; 3]; 3]; 3],
}

impl ChunkNeighborhood {
    fn at(&self, x: i32, y: i32, z: i32) -> Block {
        fn get_chunk_pos_coord(in_chunk_coord: i32) -> (usize, usize) {
            if in_chunk_coord < 0 {
                ((in_chunk_coord + CHUNK_SIZE as i32) as usize, 0)
            } else if in_chunk_coord < CHUNK_SIZE as i32 {
                (in_chunk_coord as usize, 1)
            } else {
                ((in_chunk_coord - CHUNK_SIZE as i32) as usize, 2)
            }
        }
        let (x, chunk_x) = get_chunk_pos_coord(x);
        let (y, chunk_y) = get_chunk_pos_coord(y);
        let (z, chunk_z) = get_chunk_pos_coord(z);

        return self.chunks[chunk_x][chunk_y][chunk_z]
            .as_deref()
            .map(|data| data.at(x, y, z))
            .unwrap_or_default();
    }

    pub fn at_layer(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> Block {
        let (x, y, z) = layer_to_xyz(side, layer, row, col);
        self.at(x, y, z)
    }

    pub fn middle(&self) -> Arc<ChunkData> {
        self.chunks[1][1][1].clone().unwrap()
    }

    pub fn block_is_hidden_from_above(
        &self,
        side: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> bool {
        self.at_layer(side, layer + 1, row, col) != Block::Air
    }

    pub fn count_block(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> u8 {
        if self.at_layer(side, layer, row, col) == Block::Air {
            0
        } else {
            1
        }
    }
}

pub fn layer_to_xyz(side: &BlockSide, layer: i32, row: i32, col: i32) -> (i32, i32, i32) {
    match side {
        BlockSide::Up => (row, layer, col),
        BlockSide::Down => (row, CHUNK_SIZE as i32 - 1 - layer, col),
        BlockSide::North => (layer, col, row),
        BlockSide::South => (CHUNK_SIZE as i32 - 1 - layer, row, col),
        BlockSide::East => (col, row, layer),
        BlockSide::West => (row, col, CHUNK_SIZE as i32 - 1 - layer),
    }
}
