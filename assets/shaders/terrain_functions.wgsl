#import bevy_pbr::{
    mesh_view_bindings::view,
    pbr_functions::calculate_view,
    pbr_types::{PbrInput, pbr_input_new},
}

#import "shaders/terrain_types.wgsl"::{VertexInput, VertexOutput}

@group(2) @binding(0) var texture_sampler: sampler;
@group(2) @binding(1) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(2) var overlay_textures: binding_array<texture_2d<f32>>;

const NORTH: u32 = 0;
const SOUTH: u32 = 1;
const UP: u32 = 2;
const DOWN: u32 = 3;
const EAST: u32 = 4;
const WEST: u32 = 5;

const TEXTURE_IDX_STONE: u32 = 0;
const TEXTURE_IDX_DIRT: u32 = 1;
const TEXTURE_IDX_GRASS_TOP: u32 = 2;
const TEXTURE_IDX_GRASS_SIDE: u32 = 3;
const TEXTURE_IDX_SAND: u32 = 4;
const TEXTURE_IDX_WOOD_SIDE: u32 = 5;
const TEXTURE_IDX_WOOD_TOP: u32 = 6;
const TEXTURE_IDX_OAK_LEAVES: u32 = 7;
const TEXTURE_IDX_BEDROCK: u32 = 8;
const TEXTURE_IDX_WATER: u32 = 9;

fn prepare_pbr_input(frag: VertexOutput) -> PbrInput {
    let world_normal = get_world_normal(frag.normal_id);
    var base_color = get_base_color(frag);
    // base_color += hash(vec4<f32>(floor(frag.world_position - world_normal * 0.5), 1.0)) * 0.0226;

    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.perceptual_roughness = 0.0;
    pbr_input.material.reflectance = vec3(0.0);
    pbr_input.material.base_color = base_color;

    pbr_input.frag_coord = frag.clip_position;
    pbr_input.world_position = frag.world_position;
    pbr_input.world_normal = world_normal;

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.N = normalize(world_normal);
    pbr_input.V = calculate_view(frag.world_position, pbr_input.is_orthographic);

    return pbr_input;
}

fn get_base_color(mesh: VertexOutput) -> vec4<f32> {
    let texture_index = mesh.texture_index;
    let uv = get_uv(mesh.world_position.xyz, mesh.normal_id);
    let overlay_index = get_overlay_index(mesh.texture_index);
    var color = vec4(0., 0., 0., 0.);

    // Test for 
    if (overlay_index != -1) {
        let idx = u32(overlay_index);
        color = textureSample(overlay_textures[idx], texture_sampler, uv) * get_color_for_texture(mesh.texture_index);
    }

    if color.w == 0. {
        color = textureSample(textures[mesh.texture_index], texture_sampler, uv);
        // If no overlay, assume color applies to whole texture
        if (overlay_index == -1) {
            color *= get_color_for_texture(mesh.texture_index);
        }
    }
    
    if color.w == 0. {
        discard;
    }
    
    let ao_brightness_color = vec3(mesh.ao_brightness);
    return color * vec4(ao_brightness_color, 1.0);
}

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
            return vec3(1., 0., 0.);
        }
    }
}

fn get_color_for_texture(texture_index: u32) -> vec4<f32> {
    switch (texture_index) {
        case TEXTURE_IDX_GRASS_SIDE, TEXTURE_IDX_GRASS_TOP: {
            return vec4(0.2, 0.6, 0.0, 1.0);
        }
        case TEXTURE_IDX_OAK_LEAVES: {
            return vec4(0.03, 0.295, 0.045, 1.0);
        }
        case TEXTURE_IDX_WATER: {
            return vec4(0.046, 0.184, 0.782, 0.5);
        }
        default: {
            return vec4(1., 1., 1., 1.);
        }
    }
}

fn get_overlay_index(texture_index: u32) -> i32 {
    switch (texture_index) {
        case TEXTURE_IDX_GRASS_SIDE: {
            return 0;
        }
        default: {
            return -1;
        }
    }
}