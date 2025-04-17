use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    },
};

use crate::block::{Block, BlockSide};

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<TerrainMaterial>::default(),
            MaterialPlugin::<FluidMaterial> {
                prepass_enabled: false,
                ..default()
            },
        ))
        .add_systems(Startup, setup);
    }
}

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct TerrainMaterialExtension {
    #[uniform(100)]
    quantize_steps: u32,
}

const TERRAIN_MATERIAL_SHADER_PATH: &str = "shaders/terrain.wgsl";

impl MaterialExtension for TerrainMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        TERRAIN_MATERIAL_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        TERRAIN_MATERIAL_SHADER_PATH.into()
    }
}

pub type FluidMaterial = ExtendedMaterial<StandardMaterial, FluidMaterialExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct FluidMaterialExtension {
    #[uniform(100)]
    is_translucent: u32,
    /// higher value -> "thicker" fluid which gets opaque faster
    #[uniform(101)]
    b: f32,
}

const FLUID_MATERIAL_SHADER_PATH: &str = "shaders/fluid.wgsl";

impl MaterialExtension for FluidMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        FLUID_MATERIAL_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        FLUID_MATERIAL_SHADER_PATH.into()
    }
}

#[derive(Resource)]
pub struct BlockMaterials {
    stone: Handle<TerrainMaterial>,
    dirt: Handle<TerrainMaterial>,
    grass: Handle<TerrainMaterial>,
    sand: Handle<TerrainMaterial>,
    wood: Handle<TerrainMaterial>,
    wood_top: Handle<TerrainMaterial>,
    leaves: Handle<TerrainMaterial>,
    water: Handle<FluidMaterial>,
}

impl BlockMaterials {
    pub fn get(&self, block: &Block, side: &BlockSide) -> MaterialHandle {
        match (block, side) {
            (Block::Stone, _) => MaterialHandle::Terrain(&self.stone),
            (Block::Dirt, _) => MaterialHandle::Terrain(&self.dirt),
            (Block::Grass, _) => MaterialHandle::Terrain(&self.grass),
            (Block::Sand, _) => MaterialHandle::Terrain(&self.sand),
            (Block::Wood, BlockSide::Down) | (Block::Wood, BlockSide::Up) => {
                MaterialHandle::Terrain(&self.wood_top)
            }
            (Block::Wood, _) => MaterialHandle::Terrain(&self.wood),
            (Block::Leaves, _) => MaterialHandle::Terrain(&self.leaves),
            (Block::Water, _) => MaterialHandle::Fluid(&self.water),
            _ => MaterialHandle::None,
        }
    }
}

pub enum MaterialHandle<'a> {
    None,
    Terrain(&'a Handle<TerrainMaterial>),
    Fluid(&'a Handle<FluidMaterial>),
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut fluid_materials: ResMut<Assets<FluidMaterial>>,
) {
    let mut get_material =
        |path, colour| get_material_with_colour(path, &asset_server, &mut materials, colour);

    let block_materials = BlockMaterials {
        stone: get_material("textures/blocks/stone.png", Block::Stone.get_colour()),
        dirt: get_material("textures/blocks/dirt.png", Block::Dirt.get_colour()),
        grass: get_material("textures/blocks/grass.png", Block::Grass.get_colour()),
        sand: get_material("textures/blocks/sand.png", Block::Sand.get_colour()),
        wood: get_material("textures/blocks/oak_log.png", Block::Wood.get_colour()),
        wood_top: get_material("textures/blocks/oak_log_top.png", Block::Wood.get_colour()),
        leaves: get_material("textures/blocks/oak_leaves.png", Block::Leaves.get_colour()),
        water: get_fluid_material(
            "textures/blocks/water.png",
            &asset_server,
            &mut fluid_materials,
            Block::Water.get_colour(),
        ),
    };
    commands.insert_resource(block_materials);
}

fn get_material_with_colour(
    path: &'static str,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<TerrainMaterial>>,
    colour: Color,
) -> Handle<TerrainMaterial> {
    let image = load_repeating_image_texture(path, asset_server);
    let base = StandardMaterial {
        base_color_texture: Some(image),
        base_color: colour,
        reflectance: 0.0,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    };
    let extension = TerrainMaterialExtension::default();
    return materials.add(ExtendedMaterial { base, extension });
}

fn get_fluid_material(
    path: &'static str,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<FluidMaterial>>,
    colour: Color,
) -> Handle<FluidMaterial> {
    let image = load_repeating_image_texture(path, asset_server);
    let base = StandardMaterial {
        base_color_texture: Some(image),
        base_color: colour,
        reflectance: 0.1,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    let extension = FluidMaterialExtension {
        is_translucent: 1,
        b: 0.25,
    };
    return materials.add(ExtendedMaterial { base, extension });
}

fn load_repeating_image_texture(
    path: &'static str,
    asset_server: &Res<'_, AssetServer>,
) -> Handle<Image> {
    asset_server.load_with_settings(path, |image_loader_settings| {
        *image_loader_settings = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                // rewriting mode to repeat image,
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        }
    })
}
