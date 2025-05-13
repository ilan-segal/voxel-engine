use bevy::prelude::*;

pub struct CameraDistancePlugin;

impl Plugin for CameraDistancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (assign_chunk_distance, update_chunk_distance));
    }
}

#[derive(Component, Default, PartialEq, PartialOrd)]
pub struct CameraDistance(pub f32);

impl Ord for CameraDistance {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.0.partial_cmp(&other.0).unwrap();
    }
}

impl Eq for CameraDistance {}

fn assign_chunk_distance(
    mut commands: Commands,
    q_chunk: Query<Entity, (With<GlobalTransform>, Without<CameraDistance>)>,
) {
    for e in q_chunk.iter() {
        let Ok(mut entity_commands) = commands.get_entity(e) else {
            continue;
        };
        entity_commands.try_insert(CameraDistance::default());
    }
}

fn update_chunk_distance(
    mut q_distance: Query<(&GlobalTransform, &mut CameraDistance), Without<Camera3d>>,
    q_camera: Query<&GlobalTransform, With<Camera3d>>,
) {
    let Ok(camera_transform) = q_camera.single() else {
        return;
    };
    let camera_pos = camera_transform.translation();
    for (obj_transform, mut camera_distance) in q_distance.iter_mut() {
        let obj_pos = obj_transform.translation();
        let distance = obj_pos.distance(camera_pos);
        camera_distance.0 = distance;
    }
}
