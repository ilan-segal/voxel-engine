use bevy::{color::palettes::css::BLUE, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    chunk::{position::ChunkPosition, CHUNK_SIZE},
    player::Player,
};

pub struct ChunkBorderPlugin;

impl Plugin for ChunkBorderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawChunkBorder>()
            // .add_systems(Startup, setup.after(CubeFrameSetup))
            .add_systems(
                Update,
                (
                    // update_border_position,
                    update_visibility.run_if(input_just_pressed(KeyCode::F5)),
                    draw_border.run_if(resource_equals(DrawChunkBorder(true))),
                ),
            );
    }
}

#[derive(Resource, Default, PartialEq, Eq)]
struct DrawChunkBorder(bool);

fn update_visibility(mut res_draw: ResMut<DrawChunkBorder>) {
    res_draw.0 = !res_draw.0;
}

fn draw_border(mut gizmos: Gizmos, q_chunk_pos: Query<&ChunkPosition, With<Player>>) {
    if let Ok(ChunkPosition(chunk_pos)) = q_chunk_pos.get_single() {
        let translation = (chunk_pos.as_vec3() + Vec3::splat(0.5)) * CHUNK_SIZE as f32;
        let transform =
            Transform::from_translation(translation).with_scale(Vec3::splat(CHUNK_SIZE as f32));
        gizmos.cuboid(transform, BLUE);
    }
}
