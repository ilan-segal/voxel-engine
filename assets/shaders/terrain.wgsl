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

const NORTH: u32 = 0;
const SOUTH: u32 = 1;
const UP: u32 = 2;
const DOWN: u32 = 3;
const EAST: u32 = 4;
const WEST: u32 = 5;

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
    out.world_normal = get_world_normal(normal_id);
    out.texture_index = block_id;
    out.color = vec4(get_ao_factor(ao_factor) * vec3(0.2, 0.5, 0.2), 1.0);
    return out;
}

fn get_world_normal(normal_id: u32) -> vec3<f32> {
    switch normal_id {
        case NORTH: {
            return vec3(1., 0., 0.);
        }
        case SOUTH: {
            return vec3(-1., 0., 0.);
        }
        case UP: {
            return vec3(0., 1., 0.);
        }
        case DOWN: {
            return vec3(0., -1., 0.);
        }
        case EAST: {
            return vec3(0., 0., 1.);
        }
        case WEST: {
            return vec3(0., 0., -1.);
        }
        default: {
            return vec3(0., 0., 0.);
        }
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