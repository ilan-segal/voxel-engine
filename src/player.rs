use crate::{
    block::{Block, SURFACE_HEIGHT},
    chunk::{data::Blocks, CHUNK_SIZE},
    physics::{
        aabb::Aabb, collision::Collidable, falling_state::FallingState, gravity::Gravity,
        velocity::Velocity,
    },
    render_layer::{PORTAL_LAYER, WORLD_LAYER},
    world::neighborhood::ComponentIndex,
};
use bevy::{prelude::*, render::view::RenderLayers};
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
        .add_systems(
            Update,
            (
                update_gravity,
                (update_camera_block, update_distance_fog).chain(),
            ),
        );
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
    HotbarSelection,
    PickUpRange { meters: 2.0 }
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
    // TODO: Enable DepthPrepass once implemented for terrain material
    // DepthPrepass,
    DistanceFog { ..air_distance_fog() },
    RenderLayers::from_layers(&[WORLD_LAYER, PORTAL_LAYER]),
    CameraBlock,
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

#[derive(Component, Default)]
pub struct CameraBlock(pub Block);

fn update_camera_block(
    mut q_player: Query<(&GlobalTransform, &mut CameraBlock), With<Player>>,
    blocks: Res<ComponentIndex<Blocks>>,
) {
    for (global_transform, mut head_block) in q_player.iter_mut() {
        let pos = global_transform.translation();
        let Some(block_at_pos) = blocks
            .at_pos(pos.floor().as_ivec3())
            .copied()
        else {
            continue;
        };
        let block_above = blocks
            .at_pos(
                (pos + Dir3::Y.as_vec3())
                    .floor()
                    .as_ivec3(),
            )
            .copied()
            .unwrap_or_default();
        match block_at_pos {
            Block::Water => {
                // CHECK BLOCK ABOVE IF HEAD IS ABOVE SURFACE LEVEL
                let height = pos.y;
                let height_in_block = height - height.floor();
                if height_in_block > SURFACE_HEIGHT && block_above != block_at_pos {
                    head_block.0 = Block::Air;
                } else {
                    head_block.0 = block_at_pos;
                }
            }
            _ => {
                head_block.0 = block_at_pos;
            }
        }
    }
}

fn air_distance_fog() -> DistanceFog {
    DistanceFog {
        color: Color::WHITE,
        falloff: FogFalloff::from_visibility_colors(
            CHUNK_SIZE as f32 * 5.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
            Color::WHITE, // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
            Color::linear_rgba(0.8, 0.8, 0.92, 0.3), //SKY_COLOUR.with_alpha(0.5), // atmospheric inscattering color (light gained due to scattering from the sun)
        ),
        ..default()
    }
}

fn water_distance_fog() -> DistanceFog {
    DistanceFog {
        color: Color::WHITE,
        falloff: FogFalloff::from_visibility_colors(
            CHUNK_SIZE as f32 * 1.0,
            Color::linear_rgb(0.05, 0.25, 0.45),
            Color::linear_rgb(0.05, 0.25, 0.45),
        ),
        ..default()
    }
}

fn update_distance_fog(
    mut q_player: Query<(&CameraBlock, &mut DistanceFog), Changed<CameraBlock>>,
) {
    for (CameraBlock(camera_block), mut fog) in q_player.iter_mut() {
        *fog = match camera_block {
            Block::Water => water_distance_fog(),
            _ => air_distance_fog(),
        }
    }
}
