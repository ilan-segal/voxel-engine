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
    quantize_steps: u32,
}

@group(2) @binding(100)
var<uniform> my_extended_material: TerrainMaterialExtension;

struct Vertex {
    /*
    Bits:
    0-5: X position (6 bits: 0 to 32, steps of 1)
    6-11: Y position (6 bits: 0 to 32, steps of 1)
    12-17: Z position (6 bits: 0 to 32, steps of 1)
    18-20: Normal index (3 bits: 0 to 5, indexing 6 possible normals in voxel terrain)
    21-22: Ambient occlusion factor (2 bits: 0 to 3, indexing 4 possible darkness values from ambient occlusion)
    23-31: ??? (Texture/material index??? With 9 bits to play with that allows for 2^9=512 possible textures)
    */
    @location(0) data: u32,
}

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
//     out.world_position = mesh_functions::mesh_position_local_to_world(mesh_world_from_local, vec4<f32>(vertex.position, 1.0));
//     out.position = mesh_functions::position_world_to_clip(out.world_position.xyz);
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