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
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
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

#[derive(Resource)]
pub struct BlockMaterials {
    stone: Handle<TerrainMaterial>,
    dirt: Handle<TerrainMaterial>,
    grass: Handle<TerrainMaterial>,
    wood: Handle<TerrainMaterial>,
    wood_top: Handle<TerrainMaterial>,
    leaves: Handle<TerrainMaterial>,
}

impl BlockMaterials {
    pub fn get(&self, block: &Block, side: &BlockSide) -> Option<&Handle<TerrainMaterial>> {
        match (block, side) {
            (Block::Stone, _) => Some(&self.stone),
            (Block::Dirt, _) => Some(&self.dirt),
            (Block::Grass, _) => Some(&self.grass),
            (Block::Wood, BlockSide::Down) | (Block::Wood, BlockSide::Up) => Some(&self.wood_top),
            (Block::Wood, _) => Some(&self.wood),
            (Block::Leaves, _) => Some(&self.leaves),
            _ => None,
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let mut get_material =
        |path, colour| get_material_with_colour(path, &asset_server, &mut materials, colour);

    let block_materials = BlockMaterials {
        stone: get_material("textures/blocks/stone.png", Block::Stone.get_colour()),
        dirt: get_material("textures/blocks/dirt.png", Block::Dirt.get_colour()),
        grass: get_material("textures/blocks/grass.png", Block::Grass.get_colour()),
        wood: get_material("textures/blocks/oak_log.png", Block::Wood.get_colour()),
        wood_top: get_material("textures/blocks/oak_log_top.png", Block::Wood.get_colour()),
        leaves: get_material("textures/blocks/oak_leaves.png", Block::Leaves.get_colour()),
    };
    commands.insert_resource(block_materials);
}

fn get_material_with_colour(
    path: &'static str,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<TerrainMaterial>>,
    colour: Color,
) -> Handle<TerrainMaterial> {
    let image = asset_server.load_with_settings(path, |image_loader_settings| {
        *image_loader_settings = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                // rewriting mode to repeat image,
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        }
    });
    let base = StandardMaterial {
        base_color_texture: Some(image),
        base_color: colour,
        reflectance: 0.0,
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..default()
    };
    let extension = TerrainMaterialExtension::default();
    return materials.add(ExtendedMaterial { base, extension });
}
