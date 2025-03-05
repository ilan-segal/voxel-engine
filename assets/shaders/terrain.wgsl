#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_functions,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct TerrainMaterialExtension {
    // quantize_steps: u32,
}

// @group(2) @binding(100)
// var<uniform> my_extended_material: TerrainMaterialExtension;

// struct Vertex {
//     @builtin(instance_index) instance_index: u32,
//     @location(0) position: vec3<f32>,
//     @location(1) normal: vec3<f32>,
//     @location(2) uv: vec2<f32>,
//     @location(5) color: vec4<f32>,
// };

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
//     var out: VertexOutput;

//     let mesh_world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);

//     out.instance_index = vertex.instance_index;
//     out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
//     out.position = position_world_to_clip(out.world_position.xyz);
//     out.world_normal = mesh_functions::mesh_normal_local_to_world(
//         vertex.normal,
//         vertex.instance_index
//     );
//     out.uv = vertex.uv;
//     out.color = vertex.color;

//     return out;
// }

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}