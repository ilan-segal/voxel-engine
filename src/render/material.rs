use std::num::NonZero;

use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_2d},
            AddressMode, AsBindGroup, AsBindGroupError, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BindGroupLayoutEntry, BindingResources, FilterMode,
            PreparedBindGroup, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
            ShaderRef, ShaderStages, SpecializedMeshPipelineError, TextureSampleType,
            UnpreparedBindGroup,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

/**
BYTE DATA:
* 0-5: x pos
* 6-11: y pos
* 12-17: z pos
* 18-20: normal index (range \[0, 5])
* 21-22: ambient occlusion factor (range \[0, 3])
* 23-31: block index (range: \[0, 2^12])
*/

pub const ATTRIBUTE_TERRAIN_VERTEX_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("TerrainVertexData", 37790000, VertexFormat::Uint32);

#[derive(Asset, Reflect, Debug, Clone, Default)]
pub struct TerrainMaterial {
    pub textures: Vec<Handle<Image>>,
    pub overlay_textures: Vec<Handle<Image>>,
}

const TERRAIN_MATERIAL_SHADER_PATH: &str = "shaders/terrain.wgsl";

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        TERRAIN_MATERIAL_SHADER_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        TERRAIN_MATERIAL_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::AlphaToCoverage
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[ATTRIBUTE_TERRAIN_VERTEX_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

const MAX_TEXTURE_COUNT: usize = 16;
const MAX_OVERLAY_COUNT: usize = 16;

impl AsBindGroup for TerrainMaterial {
    type Data = ();

    type Param = (SRes<RenderAssets<GpuImage>>, SRes<FallbackImage>);

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        (image_assets, fallback_image): &mut SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let fallback_image = &fallback_image.d2;
        let textures = match get_texture_views(
            &self.textures,
            MAX_TEXTURE_COUNT,
            image_assets,
            fallback_image,
        ) {
            Ok(value) => value,
            Err(value) => return value,
        };
        let overlay_textures = match get_texture_views(
            &self.overlay_textures,
            MAX_OVERLAY_COUNT,
            image_assets,
            fallback_image,
        ) {
            Ok(value) => value,
            Err(value) => return value,
        };

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("terrain_sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..default()
        });

        let bind_group = render_device.create_bind_group(
            "bindless_material_bind_group",
            layout,
            &BindGroupEntries::sequential((&sampler, &textures[..], &overlay_textures[..])),
        );

        Ok(PreparedBindGroup {
            bindings: BindingResources(vec![]),
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _layout: &BindGroupLayout,
        _render_device: &RenderDevice,
        _param: &mut SystemParamItem<'_, '_, Self::Param>,
        _force_no_bindless: bool,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        // We implement `as_bind_group`` directly because bindless texture
        // arrays can't be owned.
        // Or rather, they can be owned, but then you can't make a `&'a [&'a
        // TextureView]` from a vec of them in `get_binding()`.
        Err(AsBindGroupError::CreateBindGroupDirectly)
    }

    fn bind_group_layout_entries(_: &RenderDevice, _: bool) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        BindGroupLayoutEntries::with_indices(
            // The layout entries will only be visible in the fragment stage
            ShaderStages::FRAGMENT,
            (
                (0, sampler(SamplerBindingType::Filtering)),
                (
                    1,
                    texture_2d(TextureSampleType::Float { filterable: true })
                        .count(NonZero::<u32>::new(MAX_TEXTURE_COUNT as u32).unwrap()),
                ),
                (
                    2,
                    texture_2d(TextureSampleType::Float { filterable: true })
                        .count(NonZero::<u32>::new(MAX_OVERLAY_COUNT as u32).unwrap()),
                ),
            ),
        )
        .to_vec()
    }
}

fn get_texture_views<'a>(
    textures: &'a Vec<Handle<Image>>,
    max_count: usize,
    image_assets: &'a Res<'_, RenderAssets<GpuImage>>,
    fallback_image: &'a GpuImage,
) -> Result<
    Vec<&'a bevy::render::render_resource::WgpuTextureView>,
    std::result::Result<PreparedBindGroup<()>, AsBindGroupError>,
> {
    let mut images = vec![];
    for handle in textures.iter().take(max_count) {
        match image_assets.get(handle) {
            Some(image) => images.push(image),
            None => return Err(Err(AsBindGroupError::RetryNextUpdate)),
        }
    }
    let textures = vec![&fallback_image.texture_view; max_count];
    let mut textures: Vec<_> = textures
        .into_iter()
        .map(|texture| &**texture)
        .collect();
    for (id, image) in images.into_iter().enumerate() {
        textures[id] = &*image.texture_view;
    }
    Ok(textures)
}
