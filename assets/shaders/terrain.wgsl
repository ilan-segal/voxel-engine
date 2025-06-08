#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normal_id: u32,
    @location(2) texture_index: u32,
    @location(3) ao_brightness: f32,
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
    out.normal_id = normal_id;
    out.texture_index = block_id;
    out.ao_brightness = get_ao_brightness(ao_factor);
    return out;
}

fn get_ao_brightness(ao_index: u32) -> f32 {
    return pow(0.6, f32(ao_index));
}

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let uv = get_uv(mesh.world_position.xyz, mesh.normal_id);
    let color = textureSample(textures[mesh.texture_index], texture_sampler, uv) * mesh.ao_brightness;
    if color.w == 0. {
        discard;
    } else {
        return color;
    }
}

// TODO: May need to rotate/flip things
fn get_uv(world_position: vec3<f32>, normal_id: u32) -> vec2<f32> {
    switch normal_id {
        case NORTH: {
            return fract(vec2(world_position.z, -world_position.y));
        }
        case SOUTH: {
            return fract(vec2(-world_position.z, -world_position.y));
        }
        case UP: {
            return fract(vec2(-world_position.x, world_position.z));
        }
        case DOWN: {
            return fract(vec2(world_position.x, world_position.z));
        }
        case EAST: {
            return fract(vec2(-world_position.x, -world_position.y));
        }
        case WEST: {
            return fract(vec2(world_position.x, -world_position.y));
        }
        default: {
            return vec2(0., 0.);
        }
    }
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