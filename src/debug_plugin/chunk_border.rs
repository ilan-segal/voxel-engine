use bevy::{color::palettes::css::BLUE, input::common_conditions::input_just_pressed, prelude::*};
use bevy_polyline::prelude::{PolylineBundle, PolylineMaterial};

use crate::{
    chunk::{position::ChunkPosition, CHUNK_SIZE},
    cube_frame::{CubeFrameMeshHandle, CubeFrameSetup},
};

pub struct ChunkBorderPlugin;

impl Plugin for ChunkBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(CubeFrameSetup))
            .add_systems(
                Update,
                (
                    update_border_position,
                    update_visibility.run_if(input_just_pressed(KeyCode::F5)),
                ),
            );
    }
}

#[derive(Component)]
struct ChunkBorder;

fn setup(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mesh: Res<CubeFrameMeshHandle>,
) {
    let material = polyline_materials.add(PolylineMaterial {
        width: 2.0,
        color: BLUE.into(),
        perspective: false,
        depth_bias: -0.001,
        ..default()
    });
    commands.spawn((
        ChunkBorder,
        PolylineBundle {
            polyline: mesh.0.clone(),
            material,
            transform: Transform::from_scale(CHUNK_SIZE as f32 * Vec3::ONE),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
    ));
}

fn update_border_position(
    mut q_chunk_border: Query<&mut Transform, With<ChunkBorder>>,
    q_camera_chunk_position: Query<&ChunkPosition, (With<Camera3d>, Changed<ChunkPosition>)>,
) {
    let mut border_transform = q_chunk_border
        .get_single_mut()
        .expect("There should be exactly one chunk border object");
    let Ok(camera_chunk_pos) = q_camera_chunk_position.get_single() else {
        return;
    };
    let offset = CHUNK_SIZE as f32 * 0.5 * Vec3::ONE;
    border_transform.translation = offset + camera_chunk_pos.0.as_vec3() * CHUNK_SIZE as f32;
}

fn update_visibility(mut q_chunk_border_visibility: Query<&mut Visibility, With<ChunkBorder>>) {
    let mut visibility = q_chunk_border_visibility
        .get_single_mut()
        .expect("There should be exactly one chunk border object");
    *visibility = match *visibility {
        Visibility::Hidden => Visibility::Visible,
        _ => Visibility::Hidden,
    };
}
