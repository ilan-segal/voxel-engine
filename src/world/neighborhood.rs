use super::index::ChunkIndex;
use super::stage::Stage;
use super::WorldSet;
use crate::chunk::data::Blocks;
use crate::chunk::position::ChunkPosition;
use crate::chunk::{layer_to_xyz, CHUNK_SIZE};
use crate::utils::VolumetricRange;
use crate::{
    block::{Block, BlockSide},
    chunk::spatial::SpatiallyMapped,
};
use bevy::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

/// Represents a 3x3x3 cube of chunks
pub struct BlockNeighborhood {
    pub chunks: [[[Option<Arc<Blocks>>; 3]; 3]; 3],
}

impl BlockNeighborhood {
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Option<&Block> {
        let [x, chunk_x, y, chunk_y, z, chunk_z] = to_local_coordinates(x, y, z);

        self.chunks[chunk_x][chunk_y][chunk_z]
            .as_ref()
            .map(|bundle| bundle.at_pos([x, y, z]))
    }
}

fn to_local_coordinates(x: i32, y: i32, z: i32) -> [usize; 6] {
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
    [x, chunk_x, y, chunk_y, z, chunk_z]
}

#[derive(Component, Clone)]
pub struct GenericNeighborhood<T> {
    pub chunks: [[[Option<Arc<T>>; 3]; 3]; 3],
}

impl<T> GenericNeighborhood<T> {
    fn new(value: Arc<T>) -> Self {
        let mut chunks: [[[Option<Arc<T>>; 3]; 3]; 3] = default();
        chunks[1][1][1] = Some(value);
        return Self { chunks };
    }

    pub fn middle(&self) -> &Option<Arc<T>> {
        self.get_chunk(0, 0, 0)
    }

    pub fn iter_chunks(&self) -> impl Iterator<Item = &Option<Arc<T>>> {
        self.chunks.iter().flatten().flatten()
    }

    pub fn is_populated(&self) -> bool {
        self.iter_chunks()
            .all(|chunk| chunk.is_some())
    }

    /// 0,0,0 is middle
    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> &Option<Arc<T>> {
        let x = (x + 1) as usize;
        let y = (y + 1) as usize;
        let z = (z + 1) as usize;
        &self.chunks[x][y][z]
    }

    /// 0,0,0 is middle
    pub fn get_chunk_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Option<Arc<T>> {
        let x = (x + 1) as usize;
        let y = (y + 1) as usize;
        let z = (z + 1) as usize;
        self.chunks
            .get_mut(x)
            .expect("x range")
            .get_mut(y)
            .expect("y range")
            .get_mut(z)
            .expect("z range")
    }
}

impl<T: SpatiallyMapped<3>> GenericNeighborhood<T> {
    pub fn at(&self, x: i32, y: i32, z: i32) -> Option<&T::Item> {
        let [x, chunk_x, y, chunk_y, z, chunk_z] = to_local_coordinates(x, y, z);

        self.chunks[chunk_x][chunk_y][chunk_z]
            .as_ref()
            .map(|bundle| bundle.at_pos([x, y, z]))
    }
}

impl GenericNeighborhood<Blocks> {
    pub fn at_layer(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> Option<&Block> {
        let (x, y, z) = layer_to_xyz(side, layer, row, col);
        self.at(x, y, z)
    }

    pub fn block_is_hidden_from_above(
        &self,
        side: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> bool {
        let cur_block = self
            .at_layer(side, layer, row, col)
            .copied()
            .unwrap_or_default();
        match self.at_layer(side, layer + 1, row, col) {
            Some(block) if block == &cur_block => true,
            None | Some(&Block::Air) | Some(&Block::Leaves) => false,
            _ => true,
        }
    }

    pub fn count_block(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> u8 {
        match self.at_layer(side, layer, row, col) {
            None | Some(&Block::Air) => 0,
            _ => 1,
        }
    }
}

impl GenericNeighborhood<Stage> {
    pub fn get_lowest_stage(&self) -> Stage {
        self.iter_chunks()
            .filter_map(|chunk| chunk.clone())
            .map(|stage| *stage)
            .min()
            .unwrap_or_default()
    }
}

pub struct NeighborhoodPlugin<T>(PhantomData<T>);

impl<T> NeighborhoodPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component + Clone + Send + Sync + 'static> Plugin for NeighborhoodPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                copy_component::<T>,
                (initialize_neighborhood::<T>, update_neighborhood::<T>),
            )
                .chain()
                .after(WorldSet),
        );
    }
}

#[derive(Component)]
pub struct NeighborhoodComponentCopy<T>(pub Arc<T>);

impl<T: Clone> NeighborhoodComponentCopy<T> {
    fn new(value: &T) -> Self {
        Self(Arc::new(value.clone()))
    }
}

fn copy_component<T: Component + Clone>(
    mut commands: Commands,
    q: Query<(Entity, &T), Changed<T>>,
) {
    for (e, component) in q.iter() {
        commands
            .entity(e)
            .insert(NeighborhoodComponentCopy::new(component));
    }
}

fn initialize_neighborhood<T: Component + Clone>(
    mut commands: Commands,
    q: Query<
        (Entity, &NeighborhoodComponentCopy<T>, &ChunkPosition),
        Without<GenericNeighborhood<T>>,
    >,
    q_neighbor: Query<&NeighborhoodComponentCopy<T>>,
    index: Res<ChunkIndex>,
) {
    for (entity, component, pos) in q.iter() {
        let mut neighborhood = GenericNeighborhood::new(component.0.clone());
        VolumetricRange::new(-1..2, -1..2, -1..2)
            .filter(|(x, y, z)| x != &0 || y != &0 || z != &0)
            .for_each(|(x, y, z)| {
                let relative_pos = IVec3::new(x, y, z);
                let neighbor_pos = relative_pos + pos.0;
                let Some(neighbor) = index.entity_by_pos.get(&neighbor_pos) else {
                    return;
                };
                let Ok(other_neighborhood) = q_neighbor.get(*neighbor) else {
                    return;
                };
                *neighborhood.get_chunk_mut(x, y, z) = Some(other_neighborhood.0.clone());
            });
        commands
            .entity(entity)
            .insert(neighborhood);
    }
}

fn update_neighborhood<T: Component + Clone>(
    chunk_index: Res<ChunkIndex>,
    q_changed: Query<
        (Entity, &NeighborhoodComponentCopy<T>, &ChunkPosition),
        Changed<NeighborhoodComponentCopy<T>>,
    >,
    mut q_neighborhood: Query<&mut GenericNeighborhood<T>>,
) {
    for (center_id, component, chunk_pos) in q_changed.iter() {
        let value = Some(component.0.clone());
        if let Ok(mut neighborhood) = q_neighborhood.get_mut(center_id) {
            *neighborhood.get_chunk_mut(0, 0, 0) = value.clone();
        }
        for (x, y, z) in VolumetricRange::new(-1..2, -1..2, -1..2) {
            let offset = IVec3::new(x, y, z);
            let cur_pos = chunk_pos.0 + offset;
            let Some(neighbor) = chunk_index.entity_by_pos.get(&cur_pos) else {
                continue;
            };
            let Ok(mut neighborhood) = q_neighborhood.get_mut(*neighbor) else {
                continue;
            };
            let IVec3 {
                x: arr_x,
                y: arr_y,
                z: arr_z,
            } = -offset;
            *neighborhood.get_chunk_mut(arr_x, arr_y, arr_z) = value.clone();
        }
    }
}
