use bevy::{color::palettes::css::RED, input::common_conditions::input_just_pressed, prelude::*};
use bevy_polyline::{
    prelude::{PolylineBundle, PolylineMaterial},
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
                    align_hitbox_frame_to_axis,
                ),
            );
    }
}

fn setup(mut commands: Commands, mut polyline_materials: ResMut<Assets<PolylineMaterial>>) {
    let material = polyline_materials.add(PolylineMaterial {
        width: 5.0,
        color: RED.into(),
        perspective: true,
        ..default()
    });
    commands.insert_resource(HitboxFrameAssets { material });
}

#[derive(Resource)]
struct HitboxFrameAssets {
    material: Handle<PolylineMaterial>,
}

#[derive(Component)]
struct HitboxFrame;

#[derive(Component)]
struct HasHitboxFrame;

fn add_hitbox_frame(
    mut commands: Commands,
    query: Query<(Entity, &Aabb), Without<HasHitboxFrame>>,
    assets: Res<HitboxFrameAssets>,
    mesh: Res<CubeFrameMeshHandle>,
) {
    for (e, aabb) in query.iter() {
        commands
            .entity(e)
            .insert(HasHitboxFrame)
            .with_children(|child_builder| {
                child_builder.spawn((
                    HitboxFrame,
                    PolylineBundle {
                        polyline: mesh.0.clone_weak(),
                        material: assets.material.clone_weak(),
                        transform: Transform {
                            translation: aabb.get_centre_offset(),
                            scale: aabb.get_dimensions(),
                            ..Default::default()
                        },
                        visibility: Visibility::Hidden,
                        ..Default::default()
                    },
                ));
            });
    }
}

fn align_hitbox_frame_to_axis(
    mut q_hitbox_frame: Query<(&Parent, &mut Transform), With<HitboxFrame>>,
    q_parent_transform: Query<&GlobalTransform>,
) {
    for (parent, mut transform) in q_hitbox_frame.iter_mut() {
        let Ok(parent_global_transform) = q_parent_transform.get(parent.get()) else {
            continue;
        };
        transform.rotation = parent_global_transform
            .compute_transform()
            .rotation
            .inverse();
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
