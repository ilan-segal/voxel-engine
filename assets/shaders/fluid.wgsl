#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    view_transformations::{depth_ndc_to_view_z, position_world_to_view},
}

#import bevy_pbr::prepass_utils

@group(2) @binding(100) var<uniform> is_translucent: u32;
@group(2) @binding(101) var<uniform> b: f32;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
    @builtin(sample_index) sample_index: u32,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    if (is_translucent == 1) {
        let depth = bevy_pbr::prepass_utils::prepass_depth(in.position, sample_index);
        // Forward is -z so we gotta flip the sign
        let depth_view_distance = -bevy_pbr::view_transformations::depth_ndc_to_view_z(depth);
        let fluid_surface_view_pos = bevy_pbr::view_transformations::position_world_to_view(in.world_position.xyz);
        let fluid_surface_view_distance = -fluid_surface_view_pos.z;
        let thickness = depth_view_distance - fluid_surface_view_distance;
        out.color.w = get_fluid_opacity(out.color, thickness);
    }

    // out.color = vec4(depth, depth, depth, 1.0);

    return out;
}

// Inspired by basic applyFog from https://iquilezles.org/articles/fog/
fn get_fluid_opacity(fluid_color: vec4<f32>, thickness: f32) -> f32 {
    let base_opacity = fluid_color.w;
    let opacity_factor = 1.0 - exp(-thickness * b);
    return mix(base_opacity, 1.0, opacity_factor);
}