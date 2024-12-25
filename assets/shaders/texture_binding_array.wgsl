// #import bevy_pbr::forward_io::VertexOutput

// @group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
// @group(2) @binding(1) var nearest_sampler: sampler;
// // We can also have array of samplers
// // var samplers: binding_array<sampler>;

// @fragment
// fn fragment(
//     mesh: VertexOutput,
// ) -> @location(0) vec4<f32> {
//     // Select the texture to sample from using non-uniform uv coordinates
//     // let coords = clamp(vec2<u32>(mesh.uv * 4.0), vec2<u32>(0u), vec2<u32>(3u));
//     let coords = vec2<u32>(mesh.uv);
//     let index = coords.y * 4u + coords.x;
//     let vertical_repeat = 1u;
//     let horizontal_repeat = 2u;
//     let repeated_coords = vec2<f32>(
//         (mesh.uv.x % (1. / f32(horizontal_repeat))) * f32(horizontal_repeat),
//         (mesh.uv.y % (1. / f32(vertical_repeat))) * f32(vertical_repeat)
//     );
//     return textureSample(textures[index], nearest_sampler, repeated_coords);
// }

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions as fns,
    pbr_bindings,
}

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;

fn get_texture_sample(
    mesh: VertexOutput,
) -> vec4<f32> {
    // Select the texture to sample from using non-uniform uv coordinates
    // let coords = clamp(vec2<u32>(mesh.uv * 4.0), vec2<u32>(0u), vec2<u32>(3u));
    let coords = vec2<u32>(mesh.uv);
    let index = u32(coords.x) + 1;
    let horizontal_repeat = mesh.uv_b.x;
    let vertical_repeat = mesh.uv_b.y;
    let repeated_coords = vec2<f32>(
        (mesh.uv.x % (mesh.uv.x / horizontal_repeat)) * (horizontal_repeat / mesh.uv.x),
        (mesh.uv.y % (mesh.uv.y / vertical_repeat)) * (vertical_repeat / mesh.uv.y)
    );
    return textureSample(textures[index], nearest_sampler, repeated_coords) * mesh.color;
}

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let layer = i32(mesh.world_position.x) & 0x3;

    // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
    // the material members
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = get_texture_sample(mesh);
    pbr_input.material.reflectance = 0.0;

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);

    return fns::apply_pbr_lighting(pbr_input);
}