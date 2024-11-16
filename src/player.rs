use bevy::prelude::*;
use block_target::BlockTargetPlugin;
use falling_state::FallingState;

use crate::physics::{
    aabb::Aabb,
    collision::{Collidable, Collision},
    gravity::Gravity,
    velocity::Velocity,
    PhysicsSystemSet,
};

pub mod block_target;
pub mod falling_state;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlockTargetPlugin)
            .add_systems(
                Update,
                update_grounded_state
                    .after(PhysicsSystemSet::Act)
                    .before(PhysicsSystemSet::React),
            );
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    camera: Camera3dBundle,
    aabb: Aabb,
    collidable: Collidable,
    gravity: Gravity,
    falling_state: FallingState,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        let camera_pos = Transform::from_xyz(0.0, 80.0, 0.0);
        Self {
            player: Player,
            camera: Camera3dBundle {
                transform: camera_pos.looking_to(Vec3::X, Vec3::Y),
                ..Default::default()
            },
            aabb: Aabb::square_prism(0.35, 1.8, 1.65),
            collidable: Collidable,
            gravity: Gravity::default(),
            falling_state: FallingState::Falling,
        }
    }
}

#[derive(Component)]
pub struct Player;

fn update_grounded_state(
    mut q_state: Query<(&mut FallingState, &Velocity), With<Player>>,
    mut collisions: EventReader<Collision>,
) {
    for (mut state, v) in q_state.iter_mut() {
        if v.0.y != 0.0 {
            *state = FallingState::Falling;
        } else {
            *state = FallingState::Grounded;
        }
    }
    for Collision { entity, normal } in collisions.read() {
        let Ok((mut state, ..)) = q_state.get_mut(*entity) else {
            continue;
        };
        if normal.y > 0.0 {
            *state = FallingState::Grounded;
        }
    }
}
