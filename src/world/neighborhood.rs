use bevy::prelude::*;
use std::{marker::PhantomData, sync::Arc};

use crate::{chunk::position::ChunkPosition, utils::VolumetricRange};

use super::index::ChunkIndex;

pub struct NeighborhoodPlugin<T>(PhantomData<T>);

impl<T> NeighborhoodPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component + Clone> Plugin for NeighborhoodPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_copy::<T>,
                update_copy_to_match_component::<T>,
                add_neighborhood::<T>,
            ),
        );
    }
}

#[derive(Component)]
struct ComponentCopy<T>(Arc<T>);

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
    }
}

#[derive(Component)]
pub struct Neighborhood<T>([Option<Arc<T>>; 3 * 3 * 3]);

impl<T> Default for Neighborhood<T> {
    fn default() -> Self {
        Self([const { None }; 27])
    }
}

impl<T> Neighborhood<T> {
    /// 0,0,0 is center
    pub fn get(&self, x: i32, y: i32, z: i32) -> &Option<Arc<T>> {
        &self.0[Self::get_index(x, y, z)]
    }

    /// 0,0,0 is center
    pub fn get_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Option<Arc<T>> {
        self.0
            .get_mut(Self::get_index(x, y, z))
            .expect("index range")
    }

    fn get_index(x: i32, y: i32, z: i32) -> usize {
        (9 * (x + 1) + 3 * (y + 1) + (z + 1)) as usize
    }
}

fn add_neighborhood<T: Component + Send + Sync + 'static>(
    q: Query<(Entity, &ComponentCopy<T>, &ChunkPosition), Without<Neighborhood<T>>>,
    mut commands: Commands,
    index: Res<ChunkIndex>,
) {
    for (entity, component, pos) in q.iter() {
        let mut neighborhood = Neighborhood::<T>::default();
        *neighborhood.get_mut(0, 0, 0) = Some(component.0.clone());
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
            let IVec3 { x, y, z } = -offset;
            *neighborhood.get_mut(x, y, z) = Some(neighbor_component.0.clone());
        }
        commands
            .entity(entity)
            .insert(neighborhood);
    }
}
