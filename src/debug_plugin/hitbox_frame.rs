use bevy::{color::palettes::css::RED, input::common_conditions::input_just_pressed, prelude::*};
use bevy_polyline::{
    prelude::{PolylineBundle, PolylineMaterial, PolylineMaterialHandle},
    PolylinePlugin,
};

use crate::cube_frame::CubeFrameMeshHandle;
use crate::physics::aabb::Aabb;

pub struct AabbWireframePlugin;

impl Plugin for AabbWireframePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsVisible>()
            .add_plugins(PolylinePlugin)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    toggle_hitbox_visibility.run_if(input_just_pressed(KeyCode::F4)),
                    add_hitbox_frame,
                    align_hitbox_frame,
                ),
            );
    }
}

fn setup(mut commands: Commands, mut polyline_materials: ResMut<Assets<PolylineMaterial>>) {
    let material = PolylineMaterialHandle(polyline_materials.add(PolylineMaterial {
        width: 5.0,
        color: RED.into(),
        perspective: false,
        depth_bias: -0.001,
    }));
    commands.insert_resource(HitboxFrameAssets { material });
}

#[derive(Resource)]
struct HitboxFrameAssets {
    material: PolylineMaterialHandle,
}

#[derive(Component)]
struct HitboxFrame {
    parent: Entity,
    translation: Vec3,
    scale: Vec3,
}

#[derive(Component)]
struct HasHitboxFrame;

fn add_hitbox_frame(
    mut commands: Commands,
    query: Query<(Entity, &Aabb), Without<HasHitboxFrame>>,
    assets: Res<HitboxFrameAssets>,
    mesh: Res<CubeFrameMeshHandle>,
    is_visible: ResMut<IsVisible>,
) {
    for (e, aabb) in query.iter() {
        commands
            .entity(e)
            .try_insert(HasHitboxFrame);
        commands.spawn((
            HitboxFrame {
                parent: e,
                translation: -aabb.get_centre_offset(),
                scale: aabb.get_dimensions(),
            },
            PolylineBundle {
                polyline: mesh.0.clone(),
                material: assets.material.clone(),
                visibility: if is_visible.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                ..Default::default()
            },
        ));
    }
}

fn align_hitbox_frame(
    mut q_hitbox_frame: Query<(&HitboxFrame, &mut Transform)>,
    q_parent_transform: Query<&GlobalTransform>,
) {
    for (frame, mut transform) in q_hitbox_frame.iter_mut() {
        let Ok(parent_global_transform) = q_parent_transform.get(frame.parent) else {
            continue;
        };
        let parent_transform = parent_global_transform.compute_transform();
        transform.scale = frame.scale * parent_transform.scale;
        transform.translation = frame.translation + parent_transform.translation;
    }
}

#[derive(Resource, Default)]
struct IsVisible(bool);

fn toggle_hitbox_visibility(
    mut query: Query<&mut Visibility, With<HitboxFrame>>,
    mut is_visible: ResMut<IsVisible>,
) {
    is_visible.0 = !is_visible.0;
    for mut visibility in query.iter_mut() {
        *visibility = if is_visible.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
