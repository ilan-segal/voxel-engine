use std::{
    f32::consts::TAU,
    ops::{Add, Sub},
};

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    age::Age,
    block::Block,
    chunk::position::ChunkPosition,
    physics::{
        aabb::Aabb, collision::Collidable, friction::Friction, gravity::Gravity, velocity::Velocity,
    },
    render_layer::WORLD_LAYER,
    ui::block_icons::BlockMeshes,
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dropped_item_floating_movement)
            .add_systems(PostUpdate, add_mesh);
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Block(Block),
}

pub const STACK_LIMIT: u8 = 100;
pub const SECONDS_BEFORE_ELIGIBLE_FOR_PICKUP: f32 = 2.0;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct Quantity(pub u8);

impl Into<String> for Quantity {
    fn into(self) -> String {
        match self.0 {
            0 | 1 => "".into(),
            n => format!("{}", n),
        }
    }
}

impl Quantity {
    pub fn decrease(&mut self, amount: u8) {
        let value = self.0;
        self.0 = if value < amount { 0 } else { value - amount };
    }
}

impl Sub<Self> for Quantity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let cur = self.0;
        let other = rhs.0;
        if cur < other {
            Self(0)
        } else {
            Self(cur - other)
        }
    }
}

impl Add<Self> for Quantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self((self.0 as u16 + rhs.0 as u16).min(u8::MAX as u16) as u8)
    }
}

#[derive(Bundle)]
pub struct ItemBundle {
    pub item: Item,
    pub quantity: Quantity,
}

#[derive(Bundle)]
pub struct DroppedItemBundle {
    pub item: ItemBundle,
    pub spatial: SpatialBundle,
    pub chunk_position: ChunkPosition,
    pub aabb: Aabb,
    pub collidable: Collidable,
    pub gravity: Gravity,
    pub velocity: Velocity,
    pub friction: Friction,
    pub age: Age,
}

#[derive(Component)]
struct Meshed;

#[derive(Component)]
struct ItemMeshRoot;

fn add_mesh(
    mut commands: Commands,
    q_item: Query<(Entity, &Item), (With<GlobalTransform>, Without<Meshed>)>,
    block_meshes: Res<BlockMeshes>,
) {
    for (entity, item) in q_item.iter() {
        match item {
            Item::Block(block) => {
                commands
                    .entity(entity)
                    .insert(Meshed)
                    .with_children(|builder| {
                        builder
                            .spawn((
                                ItemMeshRoot,
                                InheritedVisibility::VISIBLE,
                                TransformBundle::default(),
                                Age::default(),
                            ))
                            .with_children(|sub_builder| {
                                if let Some(handles) = block_meshes.terrain.get(block) {
                                    for (mesh, material) in handles.iter() {
                                        sub_builder.spawn((
                                            MaterialMeshBundle {
                                                material: material.clone(),
                                                mesh: mesh.clone(),
                                                transform: Transform::from_translation(
                                                    Vec3::new(-1.0, 1.0, -1.0)
                                                        * DROPPED_ITEM_SCALE
                                                        * 2.0,
                                                ),
                                                ..default()
                                            },
                                            RenderLayers::layer(WORLD_LAYER),
                                        ));
                                    }
                                } else if let Some(handles) = block_meshes.fluid.get(block) {
                                    for (mesh, material) in handles.iter() {
                                        sub_builder.spawn((
                                            MaterialMeshBundle {
                                                material: material.clone(),
                                                mesh: mesh.clone(),
                                                transform: Transform::from_translation(
                                                    Vec3::new(-1.0, 1.0, -1.0)
                                                        * DROPPED_ITEM_SCALE
                                                        * 2.0,
                                                ),
                                                ..default()
                                            },
                                            RenderLayers::layer(WORLD_LAYER),
                                        ));
                                    }
                                };
                            });
                    });
            }
        }
    }
}

pub const DROPPED_ITEM_SCALE: f32 = 0.25;
const HEIGHT_AMPLITUDE: f32 = 0.5;
const HEIGHT_PERIOD_SECONDS: f32 = 2.5;
const ROTATION_PERIOD_SECONDS: f32 = 3.5;

fn dropped_item_floating_movement(mut q: Query<(&Age, &mut Transform), With<ItemMeshRoot>>) {
    for (age, mut transform) in q.iter_mut() {
        let t = age.seconds;
        let height = HEIGHT_AMPLITUDE * (1.0 + (t * TAU / HEIGHT_PERIOD_SECONDS).sin());
        transform.translation = Vec3 {
            y: height,
            ..default()
        };
        transform.rotation = Quat::from_axis_angle(Vec3::Y, t * TAU / ROTATION_PERIOD_SECONDS);
    }
}
