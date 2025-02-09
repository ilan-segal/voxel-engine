use super::index::ChunkIndex;
use crate::{
    block::{Block, BlockSide},
    chunk::{
        data::Blocks, layer_to_xyz, position::ChunkPosition, spatial::SpatiallyMapped, CHUNK_SIZE,
    },
    utils::VolumetricRange,
};
use bevy::prelude::*;
use std::{marker::PhantomData, sync::Arc};

pub struct NeighborhoodPlugin<T>(PhantomData<T>);

impl<T> NeighborhoodPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component + Clone> Plugin for NeighborhoodPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_neighborhood::<T>)
            .add_systems(
                PostUpdate,
                (
                    (add_copy::<T>, update_copy_to_match_component::<T>),
                    add_neighborhood::<T>,
                )
                    .chain()
                    .in_set(NeighborhoodSet),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NeighborhoodSet;

#[derive(Component)]
pub struct ComponentCopy<T>(pub Arc<T>);

fn add_copy<T: Component + Clone>(
    q: Query<(Entity, &T), Without<ComponentCopy<T>>>,
    mut commands: Commands,
) {
    for (entity, component) in q.iter() {
        let copy = Arc::new(component.clone());
        commands
            .entity(entity)
            .insert(ComponentCopy(copy));
    }
}

fn update_copy_to_match_component<T: Component + Clone>(
    mut q: Query<(&T, &mut ComponentCopy<T>), Changed<T>>,
) {
    for (component, mut copy) in q.iter_mut() {
        let value = Arc::new(component.clone());
        copy.0 = value;
        // event_writer.send(e.into());
    }
}

#[derive(Component, Clone)]
pub struct Neighborhood<T>(pub [Option<Arc<T>>; 3 * 3 * 3]);

impl<T> Default for Neighborhood<T> {
    fn default() -> Self {
        Self([const { None }; 27])
    }
}

impl<T> Neighborhood<T> {
    /// 0,0,0 is center
    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> &Option<Arc<T>> {
        &self.0[Self::get_chunk_index(x, y, z)]
    }

    /// 0,0,0 is center
    pub fn get_chunk_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Option<Arc<T>> {
        self.0
            .get_mut(Self::get_chunk_index(x, y, z))
            .expect("index range")
    }

    pub fn middle_chunk(&self) -> &Option<Arc<T>> {
        self.get_chunk(0, 0, 0)
    }

    fn get_chunk_index(x: i32, y: i32, z: i32) -> usize {
        (9 * (x + 1) + 3 * (y + 1) + (z + 1)) as usize
    }
}

impl<T: SpatiallyMapped<3>> Neighborhood<T> {
    pub fn at(&self, x: i32, y: i32, z: i32) -> Option<&T::Item> {
        let (x, chunk_x, y, chunk_y, z, chunk_z) = to_local_coordinates(x, y, z);

        self.get_chunk(chunk_x, chunk_y, chunk_z)
            .as_ref()
            .map(|blocks| blocks.at_pos([x, y, z]))
    }

    pub fn at_layer(&self, side: &BlockSide, layer: i32, row: i32, col: i32) -> Option<&T::Item> {
        let (x, y, z) = layer_to_xyz(side, layer, row, col);
        self.at(x, y, z)
    }
}

impl Neighborhood<Blocks> {
    pub fn block_is_hidden_from_above(
        &self,
        side: &BlockSide,
        layer: i32,
        row: i32,
        col: i32,
    ) -> bool {
        match self.at_layer(side, layer + 1, row, col) {
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

fn to_local_coordinates(x: i32, y: i32, z: i32) -> (usize, i32, usize, i32, usize, i32) {
    fn get_chunk_pos_coord(in_chunk_coord: i32) -> (usize, i32) {
        if in_chunk_coord < 0 {
            ((in_chunk_coord + CHUNK_SIZE as i32) as usize, -1)
        } else if in_chunk_coord < CHUNK_SIZE as i32 {
            (in_chunk_coord as usize, 0)
        } else {
            ((in_chunk_coord - CHUNK_SIZE as i32) as usize, 1)
        }
    }
    let (x, chunk_x) = get_chunk_pos_coord(x);
    let (y, chunk_y) = get_chunk_pos_coord(y);
    let (z, chunk_z) = get_chunk_pos_coord(z);
    (x, chunk_x, y, chunk_y, z, chunk_z)
}

fn add_neighborhood<T: Component + Send + Sync + 'static>(
    q: Query<(Entity, &ComponentCopy<T>, &ChunkPosition), Without<Neighborhood<T>>>,
    mut commands: Commands,
    index: Res<ChunkIndex>,
) {
    for (entity, component, pos) in q.iter() {
        let mut neighborhood = Neighborhood::<T>::default();
        *neighborhood.get_chunk_mut(0, 0, 0) = Some(component.0.clone());
        for (x, y, z) in VolumetricRange::new(-1..2, -1..2, -1..2) {
            let offset = IVec3::new(x, y, z);
            if offset == IVec3::ZERO {
                continue;
            }
            let neighbor_pos = pos.0 + offset;
            let Some(neighbor_id) = index.entity_by_pos.get(&neighbor_pos) else {
                continue;
            };
            let Ok((_, neighbor_component, _)) = q.get(*neighbor_id) else {
                continue;
            };
            *neighborhood.get_chunk_mut(x, y, z) = Some(neighbor_component.0.clone());
        }
        if let Some(mut c) = commands.get_entity(entity) {
            c.insert(neighborhood);
        }
    }
}

fn update_neighborhood<T: Component + Send + Sync + 'static>(
    index: Res<ChunkIndex>,
    q_changed_component: Query<(&ComponentCopy<T>, &ChunkPosition), Changed<ComponentCopy<T>>>,
    // mut component_update_events: EventReader<CopyUpdateEvent<T>>,
    mut q_neighborhood: Query<&mut Neighborhood<T>>,
) {
    for (component, pos) in q_changed_component.iter() {
        for (x, y, z) in VolumetricRange::new(-1..2, -1..2, -1..2) {
            let offset = IVec3::new(x, y, z);
            let cur_pos = pos.0 + offset;
            let Some(cur_id) = index.entity_by_pos.get(&cur_pos) else {
                if cur_pos == pos.0 {
                    warn!("Could not find middle chunk for getting id!");
                }
                continue;
            };
            let Ok(mut cur_neighborhood) = q_neighborhood.get_mut(*cur_id) else {
                if cur_pos == pos.0 {
                    warn!("Could not find middle chunk for mutating neighborhood!");
                }
                continue;
            };
            let IVec3 { x, y, z } = -offset;
            *cur_neighborhood.get_chunk_mut(x, y, z) = Some(component.0.clone());
        }
    }
}
