use bevy::{
    prelude::*,
    render::texture::{
        ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
    },
};

use crate::block::{Block, BlockSide};

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Resource)]
pub struct BlockMaterials {
    stone: Handle<StandardMaterial>,
    dirt: Handle<StandardMaterial>,
    grass: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
    wood_top: Handle<StandardMaterial>,
    leaves: Handle<StandardMaterial>,
}

impl BlockMaterials {
    pub fn get(&self, block: &Block, side: &BlockSide) -> Option<&Handle<StandardMaterial>> {
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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut get_material = |path| get_material(path, &asset_server, &mut materials);

    let block_materials = BlockMaterials {
        stone: get_material("textures/blocks/stone.png"),
        dirt: get_material("textures/blocks/dirt.png"),
        grass: get_material("textures/blocks/grass.png"),
        wood: get_material("textures/blocks/oak_log.png"),
        wood_top: get_material("textures/blocks/oak_log_top.png"),
        leaves: get_material("textures/blocks/oak_leaves.png"),
    };
    commands.insert_resource(block_materials);
}

fn get_material(
    path: &'static str,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
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
    let material = StandardMaterial {
        base_color_texture: Some(image),
        reflectance: 0.0,
        ..default()
    };
    return materials.add(material);
}
