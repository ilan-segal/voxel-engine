use core::f32;

use aabb::Aabb;
use bevy::{ecs::query::QueryData, prelude::*};
use collision::{Collidable, Collision};
use friction::Friction;
use gravity::Gravity;
use velocity::Velocity;

use crate::{
    chunk::{data::Blocks, position::ChunkPosition},
    utils::VolumetricRange,
    world::neighborhood::ComponentIndex,
};

pub mod aabb;
pub mod collision;
pub mod friction;
pub mod gravity;
pub mod velocity;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Collision>()
            .configure_sets(
                Update,
                PhysicsSystemSet::Act.before(PhysicsSystemSet::React),
            )
            .add_systems(
                Update,
                (
                    (
                        apply_gravity,
                        (
                            apply_velocity_with_terrain_collision,
                            apply_velocity_without_collision,
                        ),
                    )
                        .chain()
                        .in_set(PhysicsSystemSet::Act),
                    stop_velocity_from_collisions.in_set(PhysicsSystemSet::React),
                ),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSystemSet {
    /// Act according to current physical conditions such as gravity, velocity, and collision.
    Act,
    /// Process results of the `Act` phase such as collision events.
    React,
}

fn apply_gravity(mut q_object: Query<(&Gravity, &mut Velocity)>, time: Res<Time>) {
    for (g, mut v) in q_object.iter_mut() {
        v.0 += g.0 * time.delta_seconds();
    }
}

fn stop_velocity_from_collisions(
    mut q_object: Query<(&mut Velocity, Option<&Friction>)>,
    mut collisions: EventReader<Collision>,
) {
    for Collision { entity, normal } in collisions.read() {
        let Ok((mut v, friction)) = q_object.get_mut(*entity) else {
            continue;
        };
        let horizontal_velocity = v.0.with_y(0.0);
        if horizontal_velocity.length() > 0.0
            && normal.y != 0.0
            && v.0.y < 0.0
            && let Some(friction) = friction
        {
            if friction.coefficient < horizontal_velocity.length() {
                let dv_from_friction =
                    horizontal_velocity.normalize() * friction.coefficient * -1.0;
                v.0 += dv_from_friction;
            } else {
                v.0 -= horizontal_velocity;
            }
        }
        if normal.x != 0. {
            v.0.x = 0.;
        }
        if normal.y != 0. {
            v.0.y = 0.;
        }
        if normal.z != 0. {
            v.0.z = 0.;
        }
    }
}

fn apply_velocity_without_collision(
    mut q: Query<(&Velocity, &mut Transform), Without<Collidable>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (v, mut p) in q.iter_mut() {
        p.translation += v.0 * dt;
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MovingObjectQuery {
    entity: Entity,
    transform: &'static mut Transform,
    v: &'static mut Velocity,
    aabb: &'static Aabb,
    chunk_position: &'static ChunkPosition,
}

fn apply_velocity_with_terrain_collision(
    mut q_object: Query<MovingObjectQuery, With<Collidable>>,
    chunk_index: Res<ComponentIndex<Blocks>>,
    time: Res<Time>,
    mut collisions: EventWriter<Collision>,
) {
    for mut object in q_object.iter_mut() {
        let full_displacement = object.v.0 * time.delta_seconds();
        let adjusted_aabb = object
            .aabb
            .with_scale(object.transform.scale);
        let mut pos = object.transform.translation;
        pos.x += full_displacement.x;
        let collide_x = collide_with_terrain(
            &mut pos,
            &adjusted_aabb,
            &full_displacement.with_y(0.).with_z(0.),
            &chunk_index,
        );
        pos.y += full_displacement.y;
        let collide_y = collide_with_terrain(
            &mut pos,
            &adjusted_aabb,
            &full_displacement.with_x(0.).with_z(0.),
            &chunk_index,
        );
        pos.z += full_displacement.z;
        let collide_z = collide_with_terrain(
            &mut pos,
            &adjusted_aabb,
            &full_displacement.with_x(0.).with_y(0.),
            &chunk_index,
        );
        let mut collision_normal = object.v.0 * -1.0;
        if !collide_x {
            collision_normal.x = 0.;
        }
        if !collide_y {
            collision_normal.y = 0.;
        }
        if !collide_z {
            collision_normal.z = 0.;
        }
        if collision_normal != Vec3::ZERO {
            collisions.send(Collision {
                normal: Dir3::new(collision_normal).expect("Already checked for non-zero"),
                entity: object.entity,
            });
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
    index: &ComponentIndex<Blocks>,
) -> bool {
    let mut collision = false;

    // Messy but it seems solid enough
    // https://gamedev.stackexchange.com/a/199646
    let x1 = (pos.x - aabb.neg_x).floor();
    let x2 = (pos.x + aabb.x).floor();
    let y1 = (pos.y - aabb.neg_y).floor();
    let y2 = (pos.y + aabb.y).floor();
    let z1 = (pos.z - aabb.neg_z).floor();
    let z2 = (pos.z + aabb.z).floor();

    // Y-Axis
    if displacement.y < 0. && solid_block_is_in_range(index, x1, x2, y1, y1, z1, z2) {
        pos.y = (y1 + aabb.neg_y + 1.0).next_up();
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
    index: &ComponentIndex<Blocks>,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    z1: f32,
    z2: f32,
) -> bool {
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let z1 = z1 as i32;
    let x2 = x2 as i32;
    let y2 = y2 as i32;
    let z2 = z2 as i32;
    VolumetricRange::new(x1..x2 + 1, y1..y2 + 1, z1..z2 + 1).any(|(x, y, z)| {
        match index.at_pos([x, y, z]) {
            None => false,
            Some(block) => block.is_solid(),
        }
    })
}
