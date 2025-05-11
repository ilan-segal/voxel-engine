use bevy::{color::palettes::css::RED, input::common_conditions::input_just_pressed, prelude::*};

use crate::physics::aabb::Aabb;
use crate::physics::PhysicsSystemSet;

pub struct AabbWireframePlugin;

impl Plugin for AabbWireframePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsVisible>()
            .add_systems(
                Update,
                (
                    toggle_hitbox_visibility.run_if(input_just_pressed(KeyCode::F4)),
                    draw_hitboxes
                        .run_if(resource_equals(IsVisible(true)))
                        .after(PhysicsSystemSet::React),
                ),
            );
    }
}

#[derive(Resource, Default, PartialEq, Eq)]
struct IsVisible(bool);

fn toggle_hitbox_visibility(mut is_visible: ResMut<IsVisible>) {
    is_visible.0 = !is_visible.0;
}

fn draw_hitboxes(
    mut gizmos: Gizmos,
    q_hitbox: Query<(Entity, &Aabb)>,
    mut transform_params: ParamSet<(TransformHelper,)>,
) {
    for (entity, aabb) in q_hitbox.iter() {
        let Ok(entity_transform) = transform_params
            .p0()
            .compute_global_transform(entity)
        else {
            return;
        };
        let dimensions = aabb.get_dimensions() * entity_transform.scale();
        let translation = entity_transform.translation() - aabb.get_centre_offset();
        let transform = Transform::from_translation(translation).with_scale(dimensions);
        gizmos.cuboid(transform, RED);
    }
}
