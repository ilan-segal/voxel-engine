use bevy::prelude::*;
use itertools::Itertools;

use crate::{chunk::data::Blocks, world::neighborhood::ComponentIndex};

use super::Player;

pub struct BlockTargetPlugin;

impl Plugin for BlockTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TargetBlockChange>()
            .init_resource::<TargetedBlock>()
            .init_resource::<TargetedSpace>()
            .add_systems(
                Update,
                (
                    update_targeted_block,
                    update_targeted_block_outline,
                    draw_block_target,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct TargetedBlock(pub Option<IVec3>);

#[derive(Resource, Default)]
pub struct TargetedSpace(pub Option<IVec3>);

#[derive(Event)]
struct TargetBlockChange(Option<(IVec3, IVec3)>);

fn update_targeted_block(
    q_camera: Query<&GlobalTransform, (With<Camera3d>, With<Player>)>,
    mut targeted_block_change: EventWriter<TargetBlockChange>,
    index: Res<ComponentIndex<Blocks>>,
) {
    let Some(transform) = q_camera
        .get_single()
        .ok()
        .map(GlobalTransform::compute_transform)
    else {
        return;
    };
    let camera_pos = transform.translation;
    let camera_direction = transform.forward().as_vec3();
    const REACH_DISTANCE: f32 = 1000.0;
    let ts = std::iter::once(0.0)
        .chain(get_plane_distances(
            camera_pos.x,
            camera_direction.x,
            REACH_DISTANCE,
        ))
        .chain(get_plane_distances(
            camera_pos.y,
            camera_direction.y,
            REACH_DISTANCE,
        ))
        .chain(get_plane_distances(
            camera_pos.z,
            camera_direction.z,
            REACH_DISTANCE,
        ))
        .sorted_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|t| t.next_up())
        .collect::<Vec<_>>();
    // info!("{:?}", ts.len());
    for (t1, t2) in ts
        .iter()
        .map_windows(|[t1, t2]| 0.5 * (*t1 + *t2))
        .map_windows(|[t1, t2]| (*t1, *t2))
    {
        let pos = camera_pos + camera_direction * t2.next_down();
        // info!("{:?}", pos);
        let block = index
            .at_pos(pos.floor().as_ivec3())
            .cloned()
            .unwrap_or_default();
        if block.is_solid() {
            let block_pos = pos.floor().as_ivec3();
            let space_pos = (camera_pos + camera_direction * t1.next_down())
                .floor()
                .as_ivec3();
            targeted_block_change.send(TargetBlockChange(Some((space_pos, block_pos))));
            return;
        }
    }
    targeted_block_change.send(TargetBlockChange(None));
}

fn get_plane_distances(s: f32, ds: f32, max_t: f32) -> impl Iterator<Item = f32> {
    let t_per_block = ds.abs().recip();
    let first_t = if ds.is_sign_positive() {
        t_per_block * (s.ceil() - s)
    } else {
        t_per_block * (s - s.floor())
    };
    return (0..)
        .map(move |n| t_per_block * n as f32 + first_t)
        .take_while(move |t| *t <= max_t + t_per_block);
}

fn update_targeted_block_outline(
    mut targeted_block: ResMut<TargetedBlock>,
    mut targeted_space: ResMut<TargetedSpace>,
    mut targeted_block_change: EventReader<TargetBlockChange>,
) {
    for TargetBlockChange(change) in targeted_block_change.read() {
        match change {
            None => {
                targeted_block.0 = None;
                targeted_space.0 = None;
            }
            Some((space_pos, block_pos)) => {
                targeted_block.0 = Some(*block_pos);
                targeted_space.0 = Some(*space_pos);
            }
        }
    }
}

fn draw_block_target(mut gizmos: Gizmos, targeted_block: Res<TargetedBlock>) {
    if let Some(pos) = targeted_block.0 {
        let translation = pos.as_vec3() + Vec3::splat(0.5);
        let transform = Transform::from_translation(translation);
        gizmos.cuboid(transform, Color::BLACK);
    };
}
