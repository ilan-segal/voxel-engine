#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
}

#import bevy_pbr::prepass_utils

struct FluidMaterialExtension {
    is_translucent: u32,
    min_alpha: f32,
    max_alpha: f32,
    depth_to_max_alpha: f32,
}

@group(2) @binding(100) var<uniform> fluid_material: FluidMaterialExtension;

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

    // let depth = bevy_pbr::prepass_utils::prepass_depth(in.position, sample_index);
    // out.color = vec4(depth, depth, depth, 1.0);

    return out;
}