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
        app.add_systems(Update, (add_copy::<T>, update_copy_to_match_component::<T>));
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

#[derive(Component, Default)]
pub struct Neighborhood<T>(pub [Option<Arc<T>>; 27]);
