use bevy::{prelude::*, render::view::RenderLayers};

use crate::{player::Player, render_layer::WORLD_LAYER};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            align_portal_cameras.after(TransformSystem::TransformPropagate),
        )
        .add_systems(Startup, spawn_portals);
    }
}

fn spawn_portals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let portal_mesh_dimensions = Vec3::new(4.0, 4.0, 0.0);
    let rectangle = meshes.add(Cuboid::from_size(portal_mesh_dimensions));
    let white_material = materials.add(StandardMaterial::from_color(Color::WHITE));
    let portal_a = commands.spawn((
        Mesh3d(rectangle.clone()),
        MeshMaterial3d(white_material.clone()),
        Transform::from_xyz(-3.0, 1.0 + portal_mesh_dimensions.y * 0.5, 5.0),
        RenderLayers::layer(WORLD_LAYER),
    ));
    let portal_b = commands.spawn((
        Mesh3d(rectangle.clone()),
        MeshMaterial3d(white_material.clone()),
        //Transform::from_xyz(-46.0, 22.0 + portal_mesh_dimensions.y * 0.5, 20.0),
        Transform::from_xyz(4.0, 1.0 + portal_mesh_dimensions.y * 0.5, 5.0),
        RenderLayers::layer(WORLD_LAYER),
    ));
}

#[derive(Component)]
pub struct PortalEntrance {
    exit: Option<Entity>,
    exit_camera: Option<Entity>,
}

#[derive(Component)]
pub struct PortalCamera;

fn align_portal_cameras(
    q_player_camera_transform: Single<&GlobalTransform, (With<Camera3d>, With<Player>)>,
    q_portals: Query<(&PortalEntrance, &GlobalTransform), Without<Camera3d>>,
    mut q_portal_cameras: Query<&mut Transform, With<PortalCamera>>,
) {
    let eye_affine = q_player_camera_transform.affine();
    for (entrance, portal_entrance_global_transform) in q_portals.iter() {
        let PortalEntrance {
            exit: Some(portal_exit_id),
            exit_camera: Some(portal_exit_camera_id),
        } = entrance
        else {
            continue;
        };
        let Ok(portal_exit_affine) = q_portals
            .get(*portal_exit_id)
            .map(|(_, t)| t)
            .map(GlobalTransform::affine)
        else {
            continue;
        };
        let Ok(mut exit_camera_transform) = q_portal_cameras.get_mut(*portal_exit_camera_id) else {
            continue;
        };
        let portal_entrance_affine = portal_entrance_global_transform.affine();
        let exit_camera_affine = portal_exit_affine * portal_entrance_affine.inverse() * eye_affine;
        *exit_camera_transform = Transform::from_matrix(exit_camera_affine.into());
    }
}
