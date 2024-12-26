use bevy::{
    prelude::*,
    render::texture::{
        ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
    },
};

use crate::block::Block;

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
    leaves: Handle<StandardMaterial>,
}

impl BlockMaterials {
    pub fn get(&self, block: &Block) -> Option<&Handle<StandardMaterial>> {
        match block {
            Block::Stone => Some(&self.stone),
            Block::Dirt => Some(&self.dirt),
            Block::Grass => Some(&self.grass),
            Block::Wood => Some(&self.wood),
            Block::Leaves => Some(&self.leaves),
            _ => None,
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut get_material_unchecked =
        |block| get_material(block, &asset_server, &mut materials).unwrap();

    let block_materials = BlockMaterials {
        stone: get_material_unchecked(&Block::Stone),
        dirt: get_material_unchecked(&Block::Dirt),
        grass: get_material_unchecked(&Block::Grass),
        wood: get_material_unchecked(&Block::Wood),
        leaves: get_material_unchecked(&Block::Leaves),
    };
    commands.insert_resource(block_materials);
}

fn get_material(
    block: &Block,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Option<Handle<StandardMaterial>> {
    let image = get_image_handle(block, asset_server)?;
    let material = StandardMaterial {
        base_color_texture: Some(image),
        base_color: block
            .get_colour()
            .unwrap_or(Color::WHITE),
        reflectance: 0.0,
        ..default()
    };
    let handle = materials.add(material);
    Some(handle)
}

fn get_image_handle(block: &Block, asset_server: &Res<AssetServer>) -> Option<Handle<Image>> {
    let path = get_texture_path(&block)?;
    let handle = asset_server.load_with_settings(path, |image_loader_settings| {
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
    Some(handle)
}

fn get_texture_path(block: &Block) -> Option<&'static str> {
    match block {
        Block::Air => None,
        Block::Stone => Some("textures/blocks/stone.png"),
        Block::Dirt => Some("textures/blocks/dirt.png"),
        Block::Grass => Some("textures/blocks/grass.png"),
        Block::Wood => Some("textures/blocks/oak_log.png"),
        Block::Leaves => Some("textures/blocks/oak_leaves.png"),
    }
}
