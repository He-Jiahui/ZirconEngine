use crate::core::math::UVec2;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};

use super::super::atlas_texture_upload::{
    create_glyph_atlas_texture_array_resources, glyph_atlas_texture_array_spec,
    write_glyph_atlas_texture_upload_command,
};
use super::super::sdf_atlas::{distance_field_atlas_layer_count, SdfAtlasPlan};
use super::super::sdf_upload::{sdf_atlas_upload_commands, SdfAtlasUploadReport};

const SDF_ATLAS_TEXTURE_LABEL: &str = "zircon-screen-space-ui-sdf-atlas";
const SDF_ATLAS_VIEW_LABEL: &str = "zircon-screen-space-ui-sdf-atlas-view";
const MSDF_ATLAS_TEXTURE_LABEL: &str = "zircon-screen-space-ui-msdf-atlas";
const MSDF_ATLAS_VIEW_LABEL: &str = "zircon-screen-space-ui-msdf-atlas-view";

pub(super) struct DistanceFieldAtlasResources {
    sdf_texture: wgpu::Texture,
    _sdf_view: wgpu::TextureView,
    msdf_texture: wgpu::Texture,
    _msdf_view: wgpu::TextureView,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) size: UVec2,
    pub(super) sdf_page_count: u32,
    pub(super) msdf_page_count: u32,
}

impl DistanceFieldAtlasResources {
    pub(super) fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: UVec2,
        sdf_page_count: u32,
        msdf_page_count: u32,
    ) -> Self {
        let sdf = create_glyph_atlas_texture_array_resources(
            device,
            glyph_atlas_texture_array_spec(
                SDF_ATLAS_TEXTURE_LABEL,
                SDF_ATLAS_VIEW_LABEL,
                GlyphAtlasStorageFormat::R8Unorm,
                size,
                sdf_page_count.max(1),
            ),
        );
        let msdf = create_glyph_atlas_texture_array_resources(
            device,
            glyph_atlas_texture_array_spec(
                MSDF_ATLAS_TEXTURE_LABEL,
                MSDF_ATLAS_VIEW_LABEL,
                GlyphAtlasStorageFormat::Rgba8Unorm,
                size,
                msdf_page_count.max(1),
            ),
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-screen-space-ui-distance-field-bind-group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sdf.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&msdf.view),
                },
            ],
        });

        Self {
            sdf_texture: sdf.texture,
            _sdf_view: sdf.view,
            msdf_texture: msdf.texture,
            _msdf_view: msdf.view,
            bind_group,
            size,
            sdf_page_count: sdf_page_count.max(1),
            msdf_page_count: msdf_page_count.max(1),
        }
    }

    pub(super) fn page_counts(plan: &SdfAtlasPlan) -> (u32, u32) {
        (
            distance_field_atlas_layer_count(plan, GlyphAtlasFormat::Sdf),
            distance_field_atlas_layer_count(plan, GlyphAtlasFormat::Msdf),
        )
    }

    pub(super) fn matches(&self, size: UVec2, sdf_pages: u32, msdf_pages: u32) -> bool {
        self.size == size
            && self.sdf_page_count == sdf_pages.max(1)
            && self.msdf_page_count == msdf_pages.max(1)
    }

    pub(super) fn write(
        &self,
        queue: &wgpu::Queue,
        atlas_plan: &SdfAtlasPlan,
        pixels: &[u8],
        upload: &SdfAtlasUploadReport,
    ) {
        if pixels.is_empty() {
            return;
        }
        for command in sdf_atlas_upload_commands(atlas_plan, upload.clone(), pixels.len()) {
            let texture = match command.page_key.format {
                GlyphAtlasFormat::Sdf => &self.sdf_texture,
                GlyphAtlasFormat::Msdf => &self.msdf_texture,
                _ => continue,
            };
            write_glyph_atlas_texture_upload_command(queue, texture, pixels, command);
        }
    }
}
