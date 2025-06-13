use bevy::prelude::*;

use crate::player::Player;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            align_portal_cameras.after(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component)]
pub struct PortalEntrance {
    exit: Option<Entity>,
    exit_camera: Option<Entity>,
}

fn align_portal_cameras(
    q_player_camera_transform: Query<&GlobalTransform, (With<Camera3d>, With<Player>)>,
    q_portals: Query<(&PortalEntrance, &GlobalTransform), Without<Camera3d>>,
    mut q_portal_cameras: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok(eye_transform) = q_player_camera_transform
        .single()
        .map(GlobalTransform::compute_transform)
    else {
        return;
    };
    for (entrance, portal_entrance_global_transform) in q_portals.iter() {
        let PortalEntrance {
            exit: Some(portal_exit_id),
            exit_camera: Some(portal_exit_camera_id),
        } = entrance
        else {
            continue;
        };
        let Ok(exit_transform) = q_portals
            .get(*portal_exit_id)
            .map(|(_, t)| t)
            .map(GlobalTransform::compute_transform)
        else {
            continue;
        };
        let Ok(mut exit_camera_transform) = q_portal_cameras.get(*portal_exit_camera_id) else {
            continue;
        };

        let eye_transform_relative_to_entrance = portal_entrance_global_transform
            .affine()
            .inverse()
            .transform_point3(eye_transform.translation);
    }
}
