use crate::block::Block;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_2d},
            *,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};
use std::num::NonZeroU32;
use strum::IntoEnumIterator;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // GpuFeatureSupportChecker,
            MaterialPlugin::<BindlessMaterial>::default(),
        ))
        .add_systems(Startup, setup);
    }
}

const MAX_TEXTURE_COUNT: usize = 16;
// pub const ATTRIBUTE_REPEAT: MeshVertexAttribute =
//     MeshVertexAttribute::new("Repeat", 19093909, VertexFormat::Float32x2);

// struct GpuFeatureSupportChecker;

// impl Plugin for GpuFeatureSupportChecker {
//     fn build(&self, _app: &mut App) {}

//     fn finish(&self, app: &mut App) {
//         let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
//             return;
//         };

//         let render_device = render_app
//             .world()
//             .resource::<RenderDevice>();

//         // Check if the device support the required feature. If not, exit the example.
//         // In a real application, you should setup a fallback for the missing feature
//         if !render_device
//             .features()
//             .contains(WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
//         {
//             error!(
//                 "Render device doesn't support feature \
// SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, \
// which is required for texture binding arrays"
//             );
//             exit(1);
//         }
//     }
// }

#[derive(Resource)]
pub struct BlockMaterial {
    pub handle: Handle<BindlessMaterial>,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    //     ..default()
    // });

    // load 16 textures
    let textures: Vec<_> = Block::iter()
        .filter_map(get_texture_path)
        .map(|path| asset_server.load(path))
        .collect();

    let handle = materials.add(BindlessMaterial { textures });
    commands.insert_resource(BlockMaterial { handle });

    // a cube with multiple textures
    // commands.spawn(MaterialMeshBundle {
    //     mesh: meshes.add(Cuboid::default()),
    //     material: materials.add(BindlessMaterial { textures }),
    //     ..default()
    // });
}

fn get_texture_path(block: Block) -> Option<&'static str> {
    if block == Block::Air {
        return None;
    }
    let path = match block {
        Block::Air => unreachable!("Air block was already checked"),
        Block::Stone => "textures/blocks/stone.png",
        Block::Dirt => "textures/blocks/dirt.png",
        Block::Grass => "textures/blocks/grass.png",
        Block::Wood => "textures/blocks/oak_log.png",
        Block::Leaves => "textures/blocks/oak_leaves.png",
    };
    return Some(path);
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct BindlessMaterial {
    textures: Vec<Handle<Image>>,
}

impl AsBindGroup for BindlessMaterial {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        image_assets: &RenderAssets<GpuImage>,
        fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        // retrieve the render resources from handles
        let mut images = vec![];
        for handle in self
            .textures
            .iter()
            .take(MAX_TEXTURE_COUNT)
        {
            match image_assets.get(handle) {
                Some(image) => images.push(image),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }

        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_TEXTURE_COUNT];

        // convert bevy's resource types to WGPU's references
        let mut textures: Vec<_> = textures
            .into_iter()
            .map(|texture| &**texture)
            .collect();

        // fill in up to the first `MAX_TEXTURE_COUNT` textures and samplers to the arrays
        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        let bind_group = render_device.create_bind_group(
            "bindless_material_bind_group",
            layout,
            &BindGroupEntries::sequential((&textures[..], &fallback_image.sampler)),
        );

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _: &BindGroupLayout,
        _: &RenderDevice,
        _: &RenderAssets<GpuImage>,
        _: &FallbackImage,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        // we implement as_bind_group directly because
        panic!("bindless texture arrays can't be owned")
        // or rather, they can be owned, but then you can't make a `&'a [&'a TextureView]` from a vec of them in get_binding().
    }

    fn bind_group_layout_entries(_: &RenderDevice) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        BindGroupLayoutEntries::with_indices(
            // The layout entries will only be visible in the fragment stage
            ShaderStages::FRAGMENT,
            (
                // Screen texture
                //
                // @group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
                (
                    0,
                    texture_2d(TextureSampleType::Float { filterable: true })
                        .count(NonZeroU32::new(MAX_TEXTURE_COUNT as u32).unwrap()),
                ),
                // Sampler
                //
                // @group(2) @binding(1) var nearest_sampler: sampler;
                //
                // Note: as with textures, multiple samplers can also be bound
                // onto one binding slot:
                //
                // ```
                // sampler(SamplerBindingType::Filtering)
                //     .count(NonZeroU32::new(MAX_TEXTURE_COUNT as u32).unwrap()),
                // ```
                //
                // One may need to pay attention to the limit of sampler binding
                // amount on some platforms.
                (1, sampler(SamplerBindingType::Filtering)),
            ),
        )
        .to_vec()
    }
}

impl Material for BindlessMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/texture_binding_array.wgsl".into()
    }
}
