use std::f32::consts::TAU;

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
        app.add_systems(Update, (add_mesh, dropped_item_floating_movement));
    }
}

#[derive(Component)]
pub enum Item {
    Block(Block),
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Quantity {
    Finite(u8),
    Infinity,
}

impl Into<String> for Quantity {
    fn into(self) -> String {
        match self {
            Self::Infinity => "∞".into(),
            Self::Finite(n) => format!("{}", n),
        }
    }
}

impl Quantity {
    pub fn decrease(&mut self, amount: u8) {
        *self = match *self {
            Self::Infinity => Self::Infinity,
            Self::Finite(value) => {
                if value < amount {
                    Self::Finite(0)
                } else {
                    Self::Finite(value - amount)
                }
            }
        }
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
    pub transform: TransformBundle,
    pub chunk_position: ChunkPosition,
    pub aabb: Aabb,
    pub collidable: Collidable,
    pub gravity: Gravity,
    pub velocity: Velocity,
    pub friction: Friction,
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
                let Some(handles) = block_meshes.map.get(block) else {
                    return;
                };
                commands
                    .entity(entity)
                    .insert((Meshed, Visibility::Inherited))
                    .with_children(|builder| {
                        builder
                            .spawn((
                                ItemMeshRoot,
                                Visibility::Inherited,
                                TransformBundle::default(),
                                Age::default(),
                            ))
                            .with_children(|sub_builder| {
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
