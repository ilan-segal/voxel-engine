use core::f32;

use aabb::Aabb;
use bevy::{ecs::query::QueryData, prelude::*};
use collision::Collidable;
use gravity::Gravity;
use velocity::Velocity;

use crate::chunk::{index::ChunkIndex, position::ChunkPosition};

pub mod aabb;
pub mod collision;
pub mod gravity;
pub mod velocity;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_velocity,
                (apply_gravity, apply_velocity_with_terrain_collision).chain(),
            ),
        );
    }
}

fn add_velocity(mut commands: Commands, q: Query<Entity, (With<Transform>, Without<Velocity>)>) {
    q.iter().for_each(|e| {
        if let Some(mut entity_commands) = commands.get_entity(e) {
            entity_commands.insert(Velocity::default());
        }
    })
}

fn apply_gravity(mut q_object: Query<(&Gravity, &mut Velocity)>, time: Res<Time>) {
    for (g, mut v) in q_object.iter_mut() {
        v.0 += g.0 * time.delta_seconds();
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MovingObjectQuery {
    transform: &'static mut Transform,
    v: &'static mut Velocity,
    aabb: &'static Aabb,
    chunk_position: &'static ChunkPosition,
    collidable: Option<&'static Collidable>,
}

fn apply_velocity_with_terrain_collision(
    mut q_object: Query<MovingObjectQuery>,
    chunk_index: Res<ChunkIndex>,
    time: Res<Time>,
) {
    for mut object in q_object.iter_mut() {
        let full_displacement = object.v.0 * time.delta_seconds();
        let mut pos = object.transform.translation;
        pos.x += full_displacement.x;
        let collide_x = collide_with_terrain(
            &mut pos,
            &object.aabb,
            &full_displacement.with_y(0.).with_z(0.),
            &chunk_index,
        );
        pos.y += full_displacement.y;
        let collide_y = collide_with_terrain(
            &mut pos,
            &object.aabb,
            &full_displacement.with_x(0.).with_z(0.),
            &chunk_index,
        );
        pos.z += full_displacement.z;
        let collide_z = collide_with_terrain(
            &mut pos,
            &object.aabb,
            &full_displacement.with_x(0.).with_y(0.),
            &chunk_index,
        );
        if collide_x {
            object.v.0.x = 0.;
        }
        if collide_y {
            object.v.0.y = 0.;
        }
        if collide_z {
            object.v.0.z = 0.;
        }
        object.transform.translation = pos;
    }
}

/// Modify pos with terrain collision
/// Return true iff there was any collision
fn collide_with_terrain(
    pos: &mut Vec3,
    aabb: &Aabb,
    displacement: &Vec3,
    index: &ChunkIndex,
) -> bool {
    let mut collision = false;

    // Messy but it seems solid enough
    // https://gamedev.stackexchange.com/a/199646
    let x1 = (pos.x - aabb.neg_x).floor();
    let x2 = (pos.x + aabb.x).floor();
    let y1 = (pos.y - aabb.neg_y).floor() + 1.0;
    let y2 = (pos.y + aabb.y).floor();
    let z1 = (pos.z - aabb.neg_z).floor();
    let z2 = (pos.z + aabb.z).floor();

    // Y-Axis
    if displacement.y < 0. && solid_block_is_in_range(index, x1, x2, y1, y1, z1, z2) {
        pos.y = (y1 + aabb.neg_y).next_up();
        collision = true;
    }
    if displacement.y > 0. && solid_block_is_in_range(index, x1, x2, y2, y2, z1, z2) {
        pos.y = (y2 - aabb.y).next_down();
        collision = true;
    }

    // X-Axis
    if displacement.x < 0. && solid_block_is_in_range(index, x1, x1, y1, y2, z1, z2) {
        pos.x = (x1 + aabb.neg_x + 1.0).next_up();
        collision = true;
    }
    if displacement.x > 0. && solid_block_is_in_range(index, x2, x2, y1, y2, z1, z2) {
        pos.x = (x2 - aabb.x).next_down();
        collision = true;
    }

    // Z-Axis
    if displacement.z < 0. && solid_block_is_in_range(index, x1, x2, y1, y2, z1, z1) {
        pos.z = (z1 + aabb.neg_z + 1.0).next_up();
        collision = true;
    }
    if displacement.z > 0. && solid_block_is_in_range(index, x1, x2, y1, y2, z2, z2) {
        pos.z = (z2 - aabb.z).next_down();
        collision = true;
    }

    return collision;
}

fn solid_block_is_in_range(
    index: &ChunkIndex,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    z1: f32,
    z2: f32,
) -> bool {
    for x in x1 as i32..=x2 as i32 {
        for y in y1 as i32..=y2 as i32 {
            for z in z1 as i32..=z2 as i32 {
                if index
                    .at(x as f32, y as f32, z as f32)
                    .is_solid()
                {
                    return true;
                }
            }
        }
    }
    return false;
}
