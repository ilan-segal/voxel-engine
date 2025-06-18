#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
}

#import "shaders/terrain_functions.wgsl"::{
    prepare_pbr_input,
    UP
}
#import "shaders/terrain_types.wgsl"::{VertexInput, VertexOutput}

@vertex
fn vertex(
    in: VertexInput
) -> VertexOutput {
    let data = in.data;
    let local_x = (data >> 0) & 63;
    let local_y = (data >> 6) & 63;
    let local_z = (data >> 12) & 63;
    let normal_id = (data >> 18) & 7;
    let ao_factor = (data >> 21) & 3;
    let block_id = (data >> 23);

    var y_modifier = 1.;
    if normal_id == UP {
        y_modifier = 0.;
    }

    var out: VertexOutput;
    out.world_position = vec4(
        f32(local_x),
        f32(local_y) - y_modifier,
        f32(local_z),
        1.,
    );
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(in.instance_index),
        out.world_position,
    );
    return out;
}

@fragment
fn fragment(
	mesh: VertexOutput,
) {
    let pbr_input = prepare_pbr_input(mesh);
	let alpha = pbr_input.material.base_color.w;
	if alpha < 0.0 {
		discard;
	}
}