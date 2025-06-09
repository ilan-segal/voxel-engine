#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    pbr_functions::apply_pbr_lighting,
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
    out.normal_id = normal_id;
    out.texture_index = block_id;
    out.ao_brightness = get_ao_brightness(ao_factor);
    return out;
}

fn get_ao_brightness(ao_index: u32) -> f32 {
    return pow(0.6, f32(ao_index));
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let pbr_input = prepare_pbr_input(mesh);
    // let pbr_colour = tone_mapping(apply_pbr_lighting(pbr_input), view.color_grading);
    let pbr_colour = apply_pbr_lighting(pbr_input);
    return pbr_colour;
}

