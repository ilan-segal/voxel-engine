use crate::{
    chunk::CHUNK_SIZE,
    physics::{
        aabb::Aabb, collision::Collidable, falling_state::FallingState, gravity::Gravity,
        velocity::Velocity,
    },
    render_layer::WORLD_LAYER,
};
use bevy::{core_pipeline::prepass::DepthPrepass, prelude::*, render::view::RenderLayers};
use block_target::BlockTargetPlugin;
use controls::target_velocity::TargetVelocity;
use health::{Health, MaxHealth};
use inventory::{HotbarSelection, Inventory, PickUpRange};
use mode::PlayerMode;

pub mod block_target;
mod controls;
pub mod health;
pub mod inventory;
pub mod mode;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BlockTargetPlugin,
            controls::ControlsPlugin,
            inventory::InventoryPlugin,
        ))
        .add_systems(Update, update_gravity);
    }
}

const PLAYER_MAX_HEALTH: u32 = 20;

#[derive(Component)]
#[require(
    Transform,
    Aabb::square_prism(0.35, 1.8, 1.65),
    Collidable,
    Gravity,
    Velocity,
    FallingState,
    PlayerMode::NoClip,
    TargetVelocity,
    Sprinting,
    Jumping,
    Sneaking,
    Health(PLAYER_MAX_HEALTH),
    MaxHealth(PLAYER_MAX_HEALTH),
    Inventory::creative_default(),
    HotbarSelection
)]
pub struct Player;

#[derive(Component, Default)]
pub struct Sprinting(pub bool);

#[derive(Component, Default)]
pub struct Jumping(pub bool);

#[derive(Component, Default)]
pub struct Sneaking(pub bool);

#[derive(Component)]
#[require(
    Camera3d,
    Projection::from(PerspectiveProjection {
        fov: 70_f32.to_radians(),
        ..default()
    }),
    Msaa::Sample8,
    DepthPrepass,
    DistanceFog {
        color: Color::WHITE,
        falloff: FogFalloff::from_visibility_colors(
            CHUNK_SIZE as f32 * 5.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
            Color::WHITE, // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
            Color::linear_rgba(0.8, 0.8, 0.92, 0.3), //SKY_COLOUR.with_alpha(0.5), // atmospheric inscattering color (light gained due to scattering from the sun)
        ),
        ..default()
    },
)]
pub struct PlayerCamera;

fn update_gravity(
    q_player_mode: Query<(Entity, &PlayerMode), (With<Player>, Changed<PlayerMode>)>,
    mut commands: Commands,
) {
    for (entity, mode) in q_player_mode.iter() {
        match mode {
            PlayerMode::Survival => commands
                .entity(entity)
                .insert((Gravity::default(), Collidable)),
            PlayerMode::NoClip => commands
                .entity(entity)
                .remove::<(Gravity, Collidable)>(),
        };
    }
}
