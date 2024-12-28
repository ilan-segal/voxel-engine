use bevy::{prelude::*, render::view::RenderLayers};
use bevy_polyline::prelude::*;
use itertools::Itertools;

use crate::{
    cube_frame::{CubeFrameMeshHandle, CubeFrameSetup},
    render_layer::WORLD_LAYER,
    world::index::ChunkIndex,
};

use super::Player;

pub struct BlockTargetPlugin;

impl Plugin for BlockTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TargetBlockChange>()
            .init_resource::<TargetedBlock>()
            .add_systems(Startup, setup.after(CubeFrameSetup))
            .add_systems(
                Update,
                (update_targeted_block, update_targeted_block_outline).chain(),
            );
    }
}

fn setup(
    mut commands: Commands,
    cube_frame_mesh: Res<CubeFrameMeshHandle>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    let material = polyline_materials.add(PolylineMaterial {
        width: 1.5,
        color: LinearRgba::BLACK,
        depth_bias: -0.001,
        perspective: false,
    });
    commands.spawn((
        TargetedBlockOutline,
        PolylineBundle {
            polyline: cube_frame_mesh.0.clone_weak(),
            material,
            visibility: Visibility::Hidden,
            ..default()
        },
        RenderLayers::layer(WORLD_LAYER),
    ));
}

#[derive(Resource, Default)]
pub struct TargetedBlock(pub Option<IVec3>);

#[derive(Event)]
struct TargetBlockChange(Option<IVec3>);

#[derive(Component)]
struct TargetedBlockOutline;

fn update_targeted_block(
    q_camera: Query<&GlobalTransform, (With<Camera3d>, With<Player>)>,
    mut targeted_block_change: EventWriter<TargetBlockChange>,
    index: Res<ChunkIndex>,
) {
    let transform = q_camera
        .get_single()
        .ok()
        .map(GlobalTransform::compute_transform)
        .expect("There should be exactly one player camera");
    let camera_pos = transform.translation;
    let camera_direction = transform.forward().as_vec3();
    const REACH_DISTANCE: f32 = 1000.0;
    let ts = std::iter::once(&0.0)
        .chain(get_plane_distances(camera_pos.x, camera_direction.x, REACH_DISTANCE).iter())
        .chain(get_plane_distances(camera_pos.y, camera_direction.y, REACH_DISTANCE).iter())
        .chain(get_plane_distances(camera_pos.z, camera_direction.z, REACH_DISTANCE).iter())
        .sorted_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|t| t.next_up())
        .collect::<Vec<_>>();
    // info!("{:?}", ts.len());
    for t in ts
        .iter()
        .map_windows(|[t1, t2]| 0.5 * (*t1 + *t2))
    {
        let pos = camera_pos + camera_direction * t.next_down();
        // info!("{:?}", pos);
        let block = index.at(pos.x, pos.y, pos.z);
        if block.is_solid() {
            let block_pos = pos.floor().as_ivec3();
            targeted_block_change.send(TargetBlockChange(Some(block_pos)));
            return;
        }
    }
    targeted_block_change.send(TargetBlockChange(None));
}

fn get_plane_distances(s: f32, ds: f32, max_t: f32) -> Vec<f32> {
    if ds == 0.0 {
        return vec![];
    }
    let t_per_block = ds.abs().recip();
    let first_t = if ds.is_sign_positive() {
        t_per_block * (s.ceil() - s)
    } else {
        t_per_block * (s - s.floor())
    };
    return (0..)
        .map(|n| t_per_block * n as f32 + first_t)
        .take_while(|t| *t <= max_t + t_per_block)
        .collect();
}

fn update_targeted_block_outline(
    mut targeted_block: ResMut<TargetedBlock>,
    mut targeted_block_change: EventReader<TargetBlockChange>,
    mut q_targeted_block_outline: Query<
        (&mut Transform, &mut Visibility),
        With<TargetedBlockOutline>,
    >,
) {
    let (mut transform, mut visibility) = q_targeted_block_outline
        .get_single_mut()
        .expect("Block target outline should've been spawned on startup");
    for TargetBlockChange(change) in targeted_block_change.read() {
        targeted_block.0 = *change;
        match change {
            None => {
                *visibility = Visibility::Hidden;
            }
            Some(pos) => {
                *visibility = Visibility::Visible;
                transform.translation = pos.as_vec3() + Vec3::ONE * 0.5;
            }
        }
    }
}
