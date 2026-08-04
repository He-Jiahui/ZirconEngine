use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::asset::LIGHTMAP_RGBA16F_GPU_FORMAT;
use crate::core::framework::render::{
    EnvironmentExtract, LightProbeGridData, LightmapContractValidationError, RenderImageDimension,
    SH_L2_RGB_COEFFICIENT_COUNT,
};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};
use crate::graphics::types::GraphicsError;

pub(super) const LIGHT_PROBE_GRID_BINDING: u32 = 23;
pub(super) const LIGHTMAP_ATLAS_BINDING: u32 = 24;
pub(super) const LIGHTMAP_SAMPLER_BINDING: u32 = 28;
const LIGHT_PROBE_GRID_HEADER_WORDS: usize = 3;

#[derive(Clone)]
pub(in crate::graphics::scene::scene_renderer) struct LightmapGpuBindings {
    probe_grid_buffer: Arc<wgpu::Buffer>,
    atlas_resource: Option<Arc<GpuTextureResource>>,
    fallback_atlas_view: Arc<wgpu::TextureView>,
    fallback_sampler: Arc<wgpu::Sampler>,
}

impl LightmapGpuBindings {
    pub(in crate::graphics::scene::scene_renderer) fn bind_group_entries(
        &self,
    ) -> [wgpu::BindGroupEntry<'_>; 3] {
        [
            wgpu::BindGroupEntry {
                binding: LIGHT_PROBE_GRID_BINDING,
                resource: self.probe_grid_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: LIGHTMAP_ATLAS_BINDING,
                resource: wgpu::BindingResource::TextureView(self.atlas_view()),
            },
            wgpu::BindGroupEntry {
                binding: LIGHTMAP_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(self.sampler()),
            },
        ]
    }

    fn atlas_view(&self) -> &wgpu::TextureView {
        self.atlas_resource
            .as_ref()
            .map(|resource| resource.view())
            .unwrap_or(&self.fallback_atlas_view)
    }

    fn sampler(&self) -> &wgpu::Sampler {
        self.atlas_resource
            .as_ref()
            .map(|resource| resource.sampler())
            .unwrap_or(&self.fallback_sampler)
    }
}

pub(in crate::graphics::scene::scene_renderer) struct SceneLightmapResources {
    probe_grid_buffer: Arc<wgpu::Buffer>,
    atlas_texture: wgpu::Texture,
    fallback_atlas_view: Arc<wgpu::TextureView>,
    fallback_sampler: Arc<wgpu::Sampler>,
    atlas_resource: Option<Arc<GpuTextureResource>>,
    atlas_asset: Option<ResourceId>,
    light_set_generation: u64,
}

impl SceneLightmapResources {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let probe_grid_buffer = create_probe_grid_buffer(device, &disabled_probe_grid_words());
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-lightmap-atlas-fallback"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            atlas_texture.as_image_copy(),
            &[0; 8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let fallback_atlas_view =
            Arc::new(atlas_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("zircon-lightmap-atlas-fallback-view"),
                format: Some(wgpu::TextureFormat::Rgba16Float),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            }));
        let fallback_sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-lightmap-atlas-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        }));
        Self {
            probe_grid_buffer,
            atlas_texture,
            fallback_atlas_view,
            fallback_sampler,
            atlas_resource: None,
            atlas_asset: None,
            light_set_generation: 0,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn prepare(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        environment: &EnvironmentExtract,
    ) -> Result<(), GraphicsError> {
        self.prepare_probe_grid(device, environment.light_probe_grid())
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let Some(contract) = environment.baked_lighting() else {
            self.atlas_resource = None;
            self.atlas_asset = None;
            return Ok(());
        };
        contract
            .validate()
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        if self.atlas_asset == Some(contract.atlas) {
            return Ok(());
        }
        let resource = streamer.texture(Some(contract.atlas));
        if resource.id != Some(contract.atlas) {
            return Err(GraphicsError::Asset(format!(
                "lightmap atlas {} was not prepared by the resource streamer",
                contract.atlas
            )));
        }
        if resource.descriptor.format != LIGHTMAP_RGBA16F_GPU_FORMAT
            || resource.descriptor.dimension != RenderImageDimension::D2
            || resource.descriptor.array_layer_count != contract.atlas_descriptor.page_count
        {
            return Err(GraphicsError::Asset(format!(
                "lightmap atlas {} GPU descriptor does not match the consumption contract",
                contract.atlas
            )));
        }
        self.atlas_resource = Some(resource);
        self.atlas_asset = Some(contract.atlas);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn prepare_probe_grid(
        &mut self,
        device: &wgpu::Device,
        probe_grid: Option<&LightProbeGridData>,
    ) -> Result<(), LightmapContractValidationError> {
        let generation = probe_grid.map_or(0, |grid| grid.light_set_generation);
        if generation == self.light_set_generation {
            return Ok(());
        }
        let words = match probe_grid {
            Some(grid) => encode_light_probe_grid_storage(grid)?,
            None => disabled_probe_grid_words(),
        };
        self.probe_grid_buffer = create_probe_grid_buffer(device, &words);
        self.light_set_generation = generation;
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn bindings(&self) -> LightmapGpuBindings {
        let _retain_fallback_atlas = &self.atlas_texture;
        LightmapGpuBindings {
            probe_grid_buffer: Arc::clone(&self.probe_grid_buffer),
            atlas_resource: self.atlas_resource.clone(),
            fallback_atlas_view: Arc::clone(&self.fallback_atlas_view),
            fallback_sampler: Arc::clone(&self.fallback_sampler),
        }
    }
}

pub(in crate::graphics::scene::scene_renderer) fn lightmap_bind_group_layout_entries()
-> [wgpu::BindGroupLayoutEntry; 3] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: LIGHT_PROBE_GRID_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    (LIGHT_PROBE_GRID_HEADER_WORDS * std::mem::size_of::<[f32; 4]>()) as u64,
                ),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: LIGHTMAP_ATLAS_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2Array,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: LIGHTMAP_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

pub(super) fn encode_light_probe_grid_storage(
    grid: &LightProbeGridData,
) -> Result<Vec<[f32; 4]>, LightmapContractValidationError> {
    grid.validate()?;
    let probe_count = grid.sh.len() as u32;
    let mut words = Vec::with_capacity(
        LIGHT_PROBE_GRID_HEADER_WORDS + grid.sh.len() * SH_L2_RGB_COEFFICIENT_COUNT,
    );
    words.push([
        grid.bounds_min.x,
        grid.bounds_min.y,
        grid.bounds_min.z,
        grid.cell_size.x,
    ]);
    words.push([
        grid.cell_size.y,
        grid.cell_size.z,
        f32::from_bits(grid.dims[0]),
        f32::from_bits(grid.dims[1]),
    ]);
    words.push([
        f32::from_bits(grid.dims[2]),
        f32::from_bits(grid.light_set_generation as u32),
        f32::from_bits((grid.light_set_generation >> 32) as u32),
        f32::from_bits(probe_count),
    ]);
    for probe in &grid.sh {
        for coefficient in probe.coefficients() {
            words.push([coefficient.x, coefficient.y, coefficient.z, 0.0]);
        }
    }
    Ok(words)
}

fn disabled_probe_grid_words() -> Vec<[f32; 4]> {
    vec![[0.0; 4]; LIGHT_PROBE_GRID_HEADER_WORDS]
}

fn create_probe_grid_buffer(device: &wgpu::Device, words: &[[f32; 4]]) -> Arc<wgpu::Buffer> {
    Arc::new(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-light-probe-grid-storage"),
            contents: bytemuck::cast_slice(words),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }),
    )
}

#[cfg(test)]
mod tests;
