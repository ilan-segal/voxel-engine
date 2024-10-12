use crate::block::{Block, BlockSide};
use bevy::{prelude::*, utils::HashMap};

pub const CHUNK_SIZE: usize = 32;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkSpatialIndex>()
            .observe(add_chunk_to_index)
            .observe(remove_chunk_from_index);
    }
}

// 32x32x32 chunk of blocks
#[derive(Component, Clone, Copy)]
pub struct Chunk {
    // x, y, z
    pub blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
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

fn add_chunk_to_index(
    trigger: Trigger<OnAdd, ChunkPosition>,
    query: Query<&ChunkPosition, With<Chunk>>,
    mut index: ResMut<ChunkSpatialIndex>,
) {
    let entity = trigger.entity();
    let Ok(pos) = query.get(entity) else {
        return;
    };
    index.map.insert(pos.0, entity);
}

fn remove_chunk_from_index(
    trigger: Trigger<OnRemove, ChunkPosition>,
    query: Query<&ChunkPosition, With<Chunk>>,
    mut index: ResMut<ChunkSpatialIndex>,
) {
    let entity = trigger.entity();
    let Ok(pos) = query.get(entity) else {
        return;
    };
    index.map.remove(&pos.0);
}

#[derive(Resource, Default)]
pub struct ChunkSpatialIndex {
    map: HashMap<IVec3, Entity>,
}

impl ChunkSpatialIndex {
    pub fn get_neighborhood(
        &self,
        pos: &ChunkPosition,
        query: &Query<&Chunk>,
    ) -> Option<ChunkNeighborhood> {
        let middle_pos = pos.0;
        let middle_entity = self.map.get(&middle_pos)?;
        let middle = *query.get(*middle_entity).ok()?;
        let neighbor_getter = |side: BlockSide| {
            self.map
                .get(&(middle_pos + side.get_direction()))
                .and_then(|entity| query.get(*entity).ok().copied())
        };
        let neighborhood = ChunkNeighborhood {
            middle,
            up: neighbor_getter(BlockSide::Up),
            down: neighbor_getter(BlockSide::Down),
            west: neighbor_getter(BlockSide::West),
            east: neighbor_getter(BlockSide::East),
            north: neighbor_getter(BlockSide::North),
            south: neighbor_getter(BlockSide::South),
        };
        return Some(neighborhood);
    }
}

pub struct ChunkNeighborhood {
    pub middle: Chunk,
    up: Option<Chunk>,
    down: Option<Chunk>,
    north: Option<Chunk>,
    south: Option<Chunk>,
    west: Option<Chunk>,
    east: Option<Chunk>,
}

impl ChunkNeighborhood {
    fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        let size = CHUNK_SIZE as i32;
        let x_status = Status::from(x);
        let y_status = Status::from(y);
        let z_status = Status::from(z);
        match (x_status, y_status, z_status) {
            (Status::Within, Status::Within, Status::Within) => {
                self.middle.blocks[x as usize][y as usize][z as usize]
            }
            (Status::Over, Status::Within, Status::Within) => self
                .north
                .map(|c| c.blocks[(x - size) as usize][y as usize][z as usize])
                .unwrap_or_default(),
            (Status::Under, Status::Within, Status::Within) => self
                .south
                .map(|c| c.blocks[(x + size) as usize][y as usize][z as usize])
                .unwrap_or_default(),
            (Status::Within, Status::Over, Status::Within) => self
                .up
                .map(|c| c.blocks[x as usize][(y - size) as usize][z as usize])
                .unwrap_or_default(),
            (Status::Within, Status::Under, Status::Within) => self
                .down
                .map(|c| c.blocks[x as usize][(y + size) as usize][z as usize])
                .unwrap_or_default(),
            (Status::Within, Status::Within, Status::Over) => self
                .east
                .map(|c| c.blocks[x as usize][y as usize][(z - size) as usize])
                .unwrap_or_default(),
            (Status::Within, Status::Within, Status::Under) => self
                .west
                .map(|c| c.blocks[x as usize][y as usize][(z + size) as usize])
                .unwrap_or_default(),
            _ => panic!(),
        }
    }

    fn block_from_layer_coords(
        &self,
        direction: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> Block {
        let (x, y, z) = get_xyz_from_layer_indices(direction, layer, row, col);
        return self.block_at(x, y, z);
    }

    pub fn block_is_hidden_from_above(
        &self,
        direction: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> bool {
        self.block_from_layer_coords(direction, layer + 1, row, col) != Block::Air
    }
}

#[derive(PartialEq, Eq)]
enum Status {
    Under,
    Within,
    Over,
}

impl From<i32> for Status {
    fn from(value: i32) -> Self {
        if value < 0 {
            Self::Under
        } else if value < CHUNK_SIZE as i32 {
            Self::Within
        } else {
            Self::Over
        }
    }
}

fn get_xyz_from_layer_indices(
    direction: &BlockSide,
    layer: i32,
    row: i32,
    col: i32,
) -> (i32, i32, i32) {
    match direction {
        BlockSide::Up => (row, layer, col),
        BlockSide::Down => (row, CHUNK_SIZE as i32 - 1 - layer, col),
        BlockSide::North => (layer, row, col),
        BlockSide::South => (CHUNK_SIZE as i32 - 1 - layer, row, col),
        BlockSide::East => (row, col, layer),
        BlockSide::West => (row, col, CHUNK_SIZE as i32 - 1 - layer),
    }
}
