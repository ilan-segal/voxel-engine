use bevy::{core_pipeline::tonemapping::DebandDither, prelude::*};
use block_target::BlockTargetPlugin;
use falling_state::FallingState;

use crate::{
    // chunk::CHUNK_SIZE,
    physics::{
        aabb::Aabb,
        collision::{Collidable, Collision},
        gravity::Gravity,
        velocity::Velocity,
        PhysicsSystemSet,
    },
};

pub mod block_target;
mod controls;
pub mod falling_state;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BlockTargetPlugin, controls::ControlsPlugin))
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
    // fog_settings: FogSettings,
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
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 70_f32.to_radians(),
                    ..default()
                }),
                deband_dither: DebandDither::Enabled,
                ..default()
            },
            // fog_settings: FogSettings {
            //     color: Color::WHITE,
            //     falloff: FogFalloff::from_visibility_colors(
            //         CHUNK_SIZE as f32 * 16.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
            //         Color::WHITE, // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
            //         Color::linear_rgba(0.8, 0.8, 0.92, 0.3), //SKY_COLOUR.with_alpha(0.5), // atmospheric inscattering color (light gained due to scattering from the sun)
            //     ),
            //     ..default()
            // },
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
