use super::index::ChunkIndex;
use crate::{chunk::position::ChunkPosition, utils::VolumetricRange};
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
        app.add_event::<CopyUpdateEvent<T>>()
            .add_systems(
                Update,
                (
                    add_copy::<T>,
                    add_neighborhood::<T>,
                    update_neighborhood::<T>,
                    update_copy_to_match_component::<T>,
                )
                    .chain(),
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

#[derive(Event)]
struct CopyUpdateEvent<T> {
    entity: Entity,
    _phantom_data: PhantomData<T>,
}

impl<T> From<Entity> for CopyUpdateEvent<T> {
    fn from(value: Entity) -> Self {
        Self {
            entity: value,
            _phantom_data: PhantomData,
        }
    }
}

fn update_copy_to_match_component<T: Component + Clone>(
    mut q: Query<(Entity, &T, &mut ComponentCopy<T>), Changed<T>>,
    mut event_writer: EventWriter<CopyUpdateEvent<T>>,
) {
    for (e, component, mut copy) in q.iter_mut() {
        let value = Arc::new(component.clone());
        copy.0 = value;
        event_writer.send(e.into());
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
            *neighborhood.get_mut(x, y, z) = Some(neighbor_component.0.clone());
        }
        if let Some(mut c) = commands.get_entity(entity) {
            c.insert(neighborhood);
        }
    }
}

fn update_neighborhood<T: Component + Send + Sync + 'static>(
    index: Res<ChunkIndex>,
    q_changed_component: Query<(&ComponentCopy<T>, &ChunkPosition)>,
    mut component_update_events: EventReader<CopyUpdateEvent<T>>,
    mut q_neighborhood: Query<&mut Neighborhood<T>>,
) {
    for event in component_update_events.read() {
        let entity = event.entity;
        let Ok((component, pos)) = q_changed_component.get(entity) else {
            warn!("Couldn't get entity for updating neighborhoods!");
            continue;
        };
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
            *cur_neighborhood.get_mut(x, y, z) = Some(component.0.clone());
        }
    }
}
