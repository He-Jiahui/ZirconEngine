use crate::core::math::UVec2;
use crate::text::atlas::GlyphAtlasStorageFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasTextureArraySpec {
    pub(in crate::graphics::scene::scene_renderer::ui) texture_label: &'static str,
    pub(in crate::graphics::scene::scene_renderer::ui) view_label: &'static str,
    pub(in crate::graphics::scene::scene_renderer::ui) size: UVec2,
    pub(in crate::graphics::scene::scene_renderer::ui) layer_count: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::ui) usage: wgpu::TextureUsages,
}

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasTextureArrayResources {
    pub(in crate::graphics::scene::scene_renderer::ui) texture: wgpu::Texture,
    pub(in crate::graphics::scene::scene_renderer::ui) view: wgpu::TextureView,
}

impl GlyphAtlasTextureArraySpec {
    pub(in crate::graphics::scene::scene_renderer::ui) fn extent(self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.size.x.max(1),
            height: self.size.y.max(1),
            depth_or_array_layers: self.layer_count.max(1),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn matches_storage(
        self,
        storage_format: GlyphAtlasStorageFormat,
    ) -> bool {
        self.format == glyph_atlas_wgpu_texture_format(storage_format)
    }
}

impl GlyphAtlasTextureArrayResources {
    pub(in crate::graphics::scene::scene_renderer::ui) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_texture_array_spec(
    texture_label: &'static str,
    view_label: &'static str,
    storage_format: GlyphAtlasStorageFormat,
    size: UVec2,
    layer_count: u32,
) -> GlyphAtlasTextureArraySpec {
    GlyphAtlasTextureArraySpec {
        texture_label,
        view_label,
        size,
        layer_count,
        format: glyph_atlas_wgpu_texture_format(storage_format),
        usage: glyph_atlas_default_texture_usage(),
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn create_glyph_atlas_texture_array_resources(
    device: &wgpu::Device,
    spec: GlyphAtlasTextureArraySpec,
) -> GlyphAtlasTextureArrayResources {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(spec.texture_label),
        size: spec.extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: spec.format,
        usage: spec.usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(spec.view_label),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });

    GlyphAtlasTextureArrayResources { texture, view }
}

fn glyph_atlas_wgpu_texture_format(storage_format: GlyphAtlasStorageFormat) -> wgpu::TextureFormat {
    match storage_format {
        GlyphAtlasStorageFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        GlyphAtlasStorageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
    }
}

fn glyph_atlas_default_texture_usage() -> wgpu::TextureUsages {
    wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING
}
