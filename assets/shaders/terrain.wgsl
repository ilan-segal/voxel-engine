#import bevy_pbr::mesh_functions::mesh_position_local_to_clip


struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) data: u32;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) texture_index: u32,
    @location(3) color: vec3<f32>,
};

@vertex
fn vertex(
    in: VertexInput
) -> VertexOutput {
    let data = in.data;
    let local_x = (data >> 0) & 63;
    let local_y = (data >> 5) & 63;
    let local_z = (data >> 10) & 63;
    let normal_id = (data >> 15) & 15;
    let ao_factor = (data >> 18) & 3;
    let block_id = (data >> 20);

    var out: VertexOutput;
    out.world_position = vec4(f32(local_x), f32(local_y), f32(local_z), 1);
    out.clip_position = mesh_position_local_to_clip(
        out.world_position,
        vec4<f32>(out.world_position, 1.0),
    );
    out.world_normal = get_world_normal(normal_id);
    out.texture_index = block_id;
    out.color = get_ao_factor(ao_factor) * vec3(1., 1., 1.);
    return out;
}

fn get_world_normal(normal_id: u32) -> vec3<f32> {
    switch normal_id {
        case 0: return vec3(1., 0., 0.);
        case 1: return vec3(-1., 0., 0.);
        case 2: return vec3(0., 1., 0.);
        case 3: return vec3(0., -1., 0.);
        case 4: return vec3(0., 0., 1.);
        case 5: return vec3(0., 0., -1.);
        default: return vec3(0., 0., 0.);
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
    // Select the texture to sample from using non-uniform uv coordinates
    let coords = clamp(vec2<u32>(mesh.uv * 4.0), vec2<u32>(0u), vec2<u32>(3u));
    let index = coords.y * 4u + coords.x;
    let inner_uv = fract(mesh.uv * 4.0);
    return textureSample(textures[index], nearest_sampler, inner_uv);
}