use crate::{
    block::Block,
    chunk::{data::Blocks, spatial::SpatiallyMapped, Chunk, NoChunkPosition},
    render_layer::BLOCK_ICON_LAYER,
    world::stage::Stage,
};
use bevy::{
    prelude::*,
    render::{
        camera::ScalingMode,
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
    pub map: HashMap<Block, Handle<Image>>,
}

fn setup_rendered_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: -700.0,
                ..default()
            },
            transform: Transform::default().looking_to(
                Vec3 {
                    x: -1.0,
                    y: -0.00,
                    z: -0.25,
                },
                Vec3::Y,
            ),
            ..default()
        },
        RenderLayers::layer(BLOCK_ICON_LAYER),
    ));
    let mut block_icon_materials = BlockIconMaterials {
        map: HashMap::new(),
    };
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let icon_layer = RenderLayers::layer(BLOCK_ICON_LAYER);
    const STEP_BETWEEN_BLOCKS: f32 = 10.0;
    const DISTANCE_FROM_BLOCK: f32 = 2.5;
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
            Transform::from_translation(Vec3::new(STEP_BETWEEN_BLOCKS * (i as f32), 0., 0.));
        let mut blocks = Blocks::default();
        *blocks.0.at_pos_mut([0, 0, 0]) = block;
        commands.spawn((
            Chunk,
            NoChunkPosition, // To prevent interacting with the real world
            Stage::final_stage(),
            icon_layer.clone(),
            blocks,
            SpatialBundle {
                visibility: Visibility::Visible,
                transform: chunk_transform,
                ..default()
            },
        ));

        // Camera looking at block
        let camera_position = chunk_transform.translation
            + Vec3 {
                x: 1.0,
                y: 2.0_f32.powf(-0.5),
                z: 1.0,
            } * DISTANCE_FROM_BLOCK;
        let camera_transform = Transform::from_translation(camera_position).looking_at(
            chunk_transform.translation
                + Vec3 {
                    y: -1.0,
                    ..default()
                },
            Vec3::Y,
        );
        commands.spawn((
            Camera3dBundle {
                projection: OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical(2.25),
                    ..default()
                }
                .into(),
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

        block_icon_materials
            .map
            .insert(block, image_handle);
    }

    commands.insert_resource(block_icon_materials);
}
