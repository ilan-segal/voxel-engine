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

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Sprinting(pub bool);

#[derive(Component, Default)]
pub struct Jumping(pub bool);

#[derive(Component, Default)]
pub struct Sneaking(pub bool);

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    transform: Transform,
    camera_3d: Camera3d,
    projection: Projection,
    msaa: Msaa,
    depth_prepass: DepthPrepass,
    render_layers: RenderLayers,
    distance_fog: DistanceFog,
    aabb: Aabb,
    collidable: Collidable,
    gravity: Gravity,
    velocity: Velocity,
    falling_state: FallingState,
    mode: PlayerMode,
    target_velocity: TargetVelocity,
    sprinting: Sprinting,
    jumping: Jumping,
    sneaking: Sneaking,
    health: Health,
    max_health: MaxHealth,
    inventory: Inventory,
    pick_up_range: PickUpRange,
    hotbar_selection: HotbarSelection,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        let camera_pos = Transform::from_xyz(0.0, 2.0, 0.0);
        let max_health = 20;
        Self {
            player: Player,
            camera_3d: Camera3d::default(),
            projection: PerspectiveProjection {
                fov: 70_f32.to_radians(),
                ..default()
            }
            .into(),
            msaa: Msaa::Sample8,
            transform: camera_pos.looking_to(Vec3::X, Vec3::Y),
            depth_prepass: DepthPrepass,
            distance_fog: DistanceFog {
                color: Color::WHITE,
                falloff: FogFalloff::from_visibility_colors(
                    CHUNK_SIZE as f32 * 5.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::WHITE, // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::linear_rgba(0.8, 0.8, 0.92, 0.3), //SKY_COLOUR.with_alpha(0.5), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
                ..default()
            },
            aabb: Aabb::square_prism(0.35, 1.8, 1.65),
            collidable: Collidable,
            gravity: Gravity::default(),
            velocity: default(),
            falling_state: FallingState::Falling,
            mode: PlayerMode::NoClip,
            target_velocity: default(),
            sprinting: default(),
            jumping: default(),
            sneaking: default(),
            health: Health(max_health),
            max_health: MaxHealth(max_health),
            inventory: Inventory::creative_default(),
            pick_up_range: PickUpRange { meters: 2.0 },
            hotbar_selection: default(),
            render_layers: RenderLayers::layer(WORLD_LAYER),
        }
    }
}

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
