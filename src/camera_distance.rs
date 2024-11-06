use bevy::prelude::*;

pub struct CameraDistancePlugin;

impl Plugin for CameraDistancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (assign_chunk_distance, update_chunk_distance));
    }
}

#[derive(Component, Default)]
pub struct CameraDistance(pub f32);

fn assign_chunk_distance(
    mut commands: Commands,
    q_chunk: Query<Entity, (With<GlobalTransform>, Without<CameraDistance>)>,
) {
    for e in q_chunk.iter() {
        let Some(mut entity_commands) = commands.get_entity(e) else {
            continue;
        };
        entity_commands.insert(CameraDistance::default());
    }
}

fn update_chunk_distance(
    mut q_distance: Query<(&GlobalTransform, &mut CameraDistance), Without<Camera3d>>,
    q_camera: Query<&GlobalTransform, With<Camera3d>>,
) {
    let Ok(camera_transform) = q_camera.get_single() else {
        return;
    };
    let camera_pos = camera_transform.translation();
    for (obj_transform, mut camera_distance) in q_distance.iter_mut() {
        let obj_pos = obj_transform.translation();
        let distance = obj_pos.distance(camera_pos);
        camera_distance.0 = distance;
    }
}
