use crate::block::BlockSide;
use bevy::prelude::*;
use position::ChunkPosition;

pub mod data;
pub mod position;
pub mod spatial;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_chunk_position, assign_chunk_position));
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

fn assign_chunk_position(
    mut commands: Commands,
    q: Query<(Entity, &Transform), Without<ChunkPosition>>,
) {
    q.iter().for_each(|(e, t)| {
        if let Some(mut entity_commands) = commands.get_entity(e) {
            entity_commands.insert(ChunkPosition::from_world_position(&t.translation));
        }
    });
}

fn update_chunk_position(
    mut q_chunk_pos: Query<(&mut ChunkPosition, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut chunk_pos, transform) in q_chunk_pos.iter_mut() {
        let new_chunk_pos = ChunkPosition::from_world_position(&transform.translation());
        if new_chunk_pos != *chunk_pos {
            chunk_pos.0 = new_chunk_pos.0;
        }
    }
}
