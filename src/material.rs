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
            AsBindGroup, AsBindGroupError, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BindGroupLayoutEntry, BindingResources, PreparedBindGroup,
            RenderPipelineDescriptor, SamplerBindingType, ShaderRef, ShaderStages,
            SpecializedMeshPipelineError, TextureSampleType, UnpreparedBindGroup,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

/**
BYTE DATA:
* 0-4: x pos
* 5-9: y pos
* 10-14: z pos
* 15-17: normal index (range \[0, 5])
* 18-19: ambient occlusion factor (range \[0, 3])
* 20-31: block index (range: \[0, 2^12])
*/

pub const TERRAIN_VERTEX_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("TerrainVertexData", 37790000, VertexFormat::Float32x4);

#[derive(Asset, Reflect, Debug, Clone, Default)]
pub struct TerrainMaterial {
    pub textures: Vec<Handle<Image>>,
}

const TERRAIN_MATERIAL_SHADER_PATH: &str = "shaders/terrain.wgsl";

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        TERRAIN_MATERIAL_SHADER_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        TERRAIN_MATERIAL_SHADER_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[TERRAIN_VERTEX_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

const MAX_TEXTURE_COUNT: usize = 16;

impl AsBindGroup for TerrainMaterial {
    type Data = ();

    type Param = (SRes<RenderAssets<GpuImage>>, SRes<FallbackImage>);

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        (image_assets, fallback_image): &mut SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        // retrieve the render resources from handles
        let mut images = vec![];
        for handle in self.textures.iter().take(MAX_TEXTURE_COUNT) {
            match image_assets.get(handle) {
                Some(image) => images.push(image),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }

        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_TEXTURE_COUNT];

        // convert bevy's resource types to WGPU's references
        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

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
                // Screen texture
                //
                // @group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
                (
                    0,
                    texture_2d(TextureSampleType::Float { filterable: true })
                        .count(NonZero::<u32>::new(MAX_TEXTURE_COUNT as u32).unwrap()),
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
                //     .count(NonZero::<u32>::new(MAX_TEXTURE_COUNT as u32).unwrap()),
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
