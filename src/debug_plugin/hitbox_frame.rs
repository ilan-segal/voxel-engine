use bevy::{color::palettes::css::RED, input::common_conditions::input_just_pressed, prelude::*};
use bevy_polyline::{
    prelude::{Polyline, PolylineBundle, PolylineMaterial},
    PolylinePlugin,
};

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

fn setup(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    let material = polyline_materials.add(PolylineMaterial {
        width: 5.0,
        color: RED.into(),
        perspective: true,
        ..default()
    });
    /*
    A___________B
    |`          :\
    | `         : \
    Y  `        :  \
    |   C-----------D
    |   :       :   :
    E__ : _X____F   :
    `   :        \  :
     Z  :         \ :
      ` :          \:
       `G___________H
    */
    const A: Vec3 = Vec3::new(0., 1., 0.);
    const B: Vec3 = Vec3::new(1., 1., 0.);
    const C: Vec3 = Vec3::new(0., 1., 1.);
    const D: Vec3 = Vec3::new(1., 1., 1.);
    const E: Vec3 = Vec3::new(0., 0., 0.);
    const F: Vec3 = Vec3::new(1., 0., 0.);
    const G: Vec3 = Vec3::new(0., 0., 1.);
    const H: Vec3 = Vec3::new(1., 0., 1.);
    let cube_frame = polylines.add(Polyline {
        vertices: [A, B, D, C, A, E, F, B, F, H, D, H, G, C, G, E]
            .iter()
            .map(|v| *v - Vec3::ONE * 0.5)
            .collect::<_>(),
    });
    commands.insert_resource(HitboxFrameAssets {
        material,
        cube_frame,
    });
}

#[derive(Resource)]
struct HitboxFrameAssets {
    material: Handle<PolylineMaterial>,
    cube_frame: Handle<Polyline>,
}

#[derive(Component)]
struct HitboxFrame;

#[derive(Component)]
struct HasHitboxFrame;

fn add_hitbox_frame(
    mut commands: Commands,
    query: Query<(Entity, &Aabb), Without<HasHitboxFrame>>,
    assets: Res<HitboxFrameAssets>,
) {
    for (e, aabb) in query.iter() {
        commands
            .entity(e)
            .insert(HasHitboxFrame)
            .with_children(|child_builder| {
                child_builder.spawn((
                    HitboxFrame,
                    PolylineBundle {
                        polyline: assets.cube_frame.clone(),
                        material: assets.material.clone(),
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
