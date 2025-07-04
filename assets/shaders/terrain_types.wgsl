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
    @location(4) world_position_offset: vec3<f32>,
};

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