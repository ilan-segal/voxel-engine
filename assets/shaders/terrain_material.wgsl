#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;

// struct TerrainMaterial {
//     color: vec4<f32>,
// };
// @group(2) @binding(0) var<uniform> material: TerrainMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) blend_color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) texture_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) blend_color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) texture_index: u32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    out.blend_color = vertex.blend_color;
    out.uv = vertex.uv;
    out.texture_index = vertex.texture_index;
    return out;
}

struct FragmentInput {
    @location(0) blend_color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) texture_index: u32,
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let color = input.blend_color * textureSample(textures[input.texture_index], nearest_sampler, fract(input.uv));
    if color.a < 0.5 {
        discard;
    }
    return color;
}