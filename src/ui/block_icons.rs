use crate::{
    block::Block,
    chunk::{data::Blocks, spatial::SpatiallyMapped, Chunk, NoChunkPosition},
    render_layer::BLOCK_ICON_LAYER,
};
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    utils::hashbrown::HashMap,
};
use strum::IntoEnumIterator;

pub struct BlockIconPlugin;

impl Plugin for BlockIconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_rendered_icons);
    }
}

#[derive(Resource)]
pub struct BlockIconMaterials {
    pub map: HashMap<Block, Handle<StandardMaterial>>,
}

fn setup_rendered_icons(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut block_icon_materials = BlockIconMaterials {
        map: HashMap::new(),
    };
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let icon_layer = RenderLayers::layer(BLOCK_ICON_LAYER);
    const STEP_BETWEEN_BLOCKS: usize = 1;
    const DISTANCE_FROM_BLOCK: f32 = 2.0;
    for (i, block) in Block::iter().enumerate() {
        // Rendering the block to this image
        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );
        // You need to set these texture usage flags in order to use the image as a render target
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

        let image_handle = images.add(image);

        // Chunk containing the block to be rendered
        let chunk_transform =
            Transform::from_translation(Vec3::new((STEP_BETWEEN_BLOCKS * i) as f32, 0., 0.));
        let mut blocks = Blocks::default();
        *blocks.0.at_pos_mut([0, 0, 0]) = block;
        commands.spawn((
            Chunk,
            NoChunkPosition, // To prevent interacting with the real world
            icon_layer.clone(),
            chunk_transform,
            blocks,
        ));

        // Camera looking at block
        let camera_position = chunk_transform.translation + Vec3::ONE * DISTANCE_FROM_BLOCK;
        let camera_transform =
            Transform::from_translation(camera_position).looking_at(Vec3::ZERO, Vec3::Y);
        commands.spawn((
            Camera3dBundle {
                projection: OrthographicProjection::default().into(),
                transform: camera_transform,
                camera: Camera {
                    target: image_handle.clone().into(),
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
            icon_layer.clone(),
        ));

        // Material containing the rendered image of the block
        let material = StandardMaterial {
            base_color_texture: Some(image_handle),
            reflectance: 0.0,
            unlit: false,
            ..default()
        };
        let material_handle = materials.add(material);
        block_icon_materials
            .map
            .insert(block, material_handle);
    }
    commands.insert_resource(block_icon_materials);
}
