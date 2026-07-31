use crate::core::math::UVec2;
use crate::text::atlas::GlyphAtlasStorageFormat;
use crate::text::atlas::render_gpu_plan::{
    GlyphAtlasGpuBindGroupLayout, GlyphAtlasGpuSamplerBindingType, GlyphAtlasGpuTextureSampleType,
    GlyphAtlasGpuTextureViewDimension, glyph_atlas_gpu_bind_group_layout,
};

use super::super::atlas_texture_upload::{
    create_glyph_atlas_texture_array_resources, glyph_atlas_texture_array_spec,
};

const GLYPH_ATLAS_TEXTURE_LABEL: &str = "zircon-screen-space-ui-glyph-atlas";
const GLYPH_ATLAS_TEXTURE_VIEW_LABEL: &str = "zircon-screen-space-ui-glyph-atlas-view";

pub(super) struct GlyphAtlasBitmapAtlasResources {
    texture: wgpu::Texture,
    _view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    size: UVec2,
    layer_count: u32,
    storage_format: GlyphAtlasStorageFormat,
}

impl GlyphAtlasBitmapAtlasResources {
    pub(super) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(super) fn size(&self) -> UVec2 {
        self.size
    }

    pub(super) fn layer_count(&self) -> u32 {
        self.layer_count
    }

    pub(super) fn storage_format(&self) -> GlyphAtlasStorageFormat {
        self.storage_format
    }

    pub(super) fn supports(
        &self,
        size: UVec2,
        required_layer_count: u32,
        storage_format: GlyphAtlasStorageFormat,
    ) -> bool {
        self.size == size
            && self.layer_count >= required_layer_count.max(1)
            && self.storage_format == storage_format
    }
}

pub(super) fn create_glyph_atlas_bitmap_bind_group_layout(
    device: &wgpu::Device,
    layout: GlyphAtlasGpuBindGroupLayout,
) -> wgpu::BindGroupLayout {
    let entries = glyph_atlas_wgpu_bind_group_layout_entries(layout);
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-screen-space-ui-glyph-atlas-bind-group-layout"),
        entries: &entries,
    })
}

pub(super) fn create_glyph_atlas_bitmap_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&glyph_atlas_bitmap_sampler_descriptor())
}

pub(super) fn glyph_atlas_bitmap_sampler_descriptor() -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("zircon-screen-space-ui-glyph-atlas-sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        ..Default::default()
    }
}

pub(super) fn create_glyph_atlas_bitmap_atlas_resources(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    viewport_uniform_buffer: &wgpu::Buffer,
    size: UVec2,
    layer_count: u32,
    storage_format: GlyphAtlasStorageFormat,
) -> GlyphAtlasBitmapAtlasResources {
    let layer_count = layer_count.max(1);
    let layout = glyph_atlas_gpu_bind_group_layout();
    let spec = glyph_atlas_texture_array_spec(
        GLYPH_ATLAS_TEXTURE_LABEL,
        GLYPH_ATLAS_TEXTURE_VIEW_LABEL,
        storage_format,
        size,
        layer_count,
    );
    let resources = create_glyph_atlas_texture_array_resources(device, spec);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-screen-space-ui-glyph-atlas-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: layout.atlas_texture.binding,
                resource: wgpu::BindingResource::TextureView(&resources.view),
            },
            wgpu::BindGroupEntry {
                binding: layout.atlas_sampler.binding,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: layout.viewport_uniform.binding,
                resource: viewport_uniform_buffer.as_entire_binding(),
            },
        ],
    });

    GlyphAtlasBitmapAtlasResources {
        texture: resources.texture,
        _view: resources.view,
        bind_group,
        size,
        layer_count,
        storage_format,
    }
}

pub(super) fn glyph_atlas_wgpu_bind_group_layout_entries(
    layout: GlyphAtlasGpuBindGroupLayout,
) -> [wgpu::BindGroupLayoutEntry; 3] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: layout.atlas_texture.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: glyph_atlas_wgpu_texture_sample_type(layout.atlas_texture.sample_type),
                view_dimension: glyph_atlas_wgpu_texture_view_dimension(
                    layout.atlas_texture.view_dimension,
                ),
                multisampled: layout.atlas_texture.multisampled,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: layout.atlas_sampler.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(glyph_atlas_wgpu_sampler_binding_type(
                layout.atlas_sampler.binding_type,
            )),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: layout.viewport_uniform.binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(16),
            },
            count: None,
        },
    ]
}

fn glyph_atlas_wgpu_texture_sample_type(
    sample_type: GlyphAtlasGpuTextureSampleType,
) -> wgpu::TextureSampleType {
    match sample_type {
        GlyphAtlasGpuTextureSampleType::FloatFilterable => {
            wgpu::TextureSampleType::Float { filterable: true }
        }
    }
}

fn glyph_atlas_wgpu_texture_view_dimension(
    dimension: GlyphAtlasGpuTextureViewDimension,
) -> wgpu::TextureViewDimension {
    match dimension {
        GlyphAtlasGpuTextureViewDimension::D2Array => wgpu::TextureViewDimension::D2Array,
    }
}

fn glyph_atlas_wgpu_sampler_binding_type(
    binding_type: GlyphAtlasGpuSamplerBindingType,
) -> wgpu::SamplerBindingType {
    match binding_type {
        GlyphAtlasGpuSamplerBindingType::Filtering => wgpu::SamplerBindingType::Filtering,
    }
}
