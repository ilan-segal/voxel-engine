#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) texture_index: u32,
    @location(3) color: vec4<f32>,
};

@vertex
fn vertex(
    in: VertexInput
) -> VertexOutput {
    let data = in.data;
    let local_x = (data >> 0) & 63;
    let local_y = (data >> 6) & 63;
    let local_z = (data >> 12) & 63;
    let normal_id = (data >> 18) & 15;
    let ao_factor = (data >> 21) & 3;
    let block_id = (data >> 23);

    var out: VertexOutput;
    out.world_position = vec4(
        f32(local_x),
        f32(local_y),
        f32(local_z),
        1.,
    );
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(in.instance_index),
        out.world_position,
    );
    out.world_normal = get_world_normal(normal_id);
    out.texture_index = block_id;
    out.color = vec4(get_ao_factor(ao_factor) * vec3(0.2, 0.5, 0.2), 1.0);
    return out;
}

fn get_world_normal(normal_id: u32) -> vec3<f32> {
    switch normal_id {
        case 0u: {return vec3(1., 0., 0.);}
        case 1u: {return vec3(-1., 0., 0.);}
        case 2u: {return vec3(0., 1., 0.);}
        case 3u: {return vec3(0., -1., 0.);}
        case 4u: {return vec3(0., 0., 1.);}
        case 5u: {return vec3(0., 0., -1.);}
        default: {return vec3(0., 0., 0.);}
    }
}

fn get_ao_factor(ao_index: u32) -> f32 {
    return pow(0.6, f32(ao_index));
}

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;
// We can also have array of samplers
// var samplers: binding_array<sampler>;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return mesh.color;
}