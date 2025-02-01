#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) texture_index: u32,
};

struct TerrainVertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) texture_index: u32,
}

fn affine_from_3d(pos: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(pos, 1.0);
}

@vertex
fn vertex(vertex: Vertex) -> TerrainVertexOutput {
    var out: TerrainVertexOutput;
    let world_from_local_affine = get_world_from_local(vertex.instance_index);
    out.world_position = vec4<f32>(vertex.position, 1.0);
    out.position = mesh_position_local_to_clip(
        world_from_local_affine,
        affine_from_3d(vertex.position),
    );
    out.world_normal = (world_from_local_affine * affine_from_3d(vertex.normal)).xyz;
    out.color = vertex.color;
    out.uv = vertex.uv;
    out.texture_index = vertex.texture_index;
    return out;
}

struct FragmentInput {
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) texture_index: u32,
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let color = input.color * textureSample(textures[input.texture_index], nearest_sampler, fract(input.uv));
    if color.a < 0.5 {
        discard;
    }
    return color;
}