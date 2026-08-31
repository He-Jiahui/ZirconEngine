use crate::core::math::UVec2;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};
use crate::text::sdf::SdfAtlasBakePage;
use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

use super::super::atlas_texture_upload::{
    create_glyph_atlas_texture_array_resources, glyph_atlas_texture_array_spec,
    glyph_atlas_texture_upload_region, glyph_atlas_texture_upload_source_range,
    glyph_atlas_texture_upload_write,
};
use super::super::sdf_atlas::{SdfAtlasPlan, distance_field_atlas_layer_count};
use super::super::sdf_upload::{SdfAtlasUploadReport, sdf_atlas_upload_commands};

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

    pub(super) fn prepare_uploads(
        &self,
        atlas_plan: &SdfAtlasPlan,
        pages: &[SdfAtlasBakePage],
        upload: &SdfAtlasUploadReport,
        texture_uploads: &mut WgpuTextureUploadBatch,
    ) -> bool {
        if upload.mode == super::super::sdf_upload::SdfAtlasUploadMode::None {
            return sdf_atlas_upload_batch_is_complete(pages, upload, &[]);
        }
        let Some(source_byte_len) = pages
            .iter()
            .try_fold(0_usize, |bytes, page| bytes.checked_add(page.byte_len))
        else {
            return false;
        };
        if source_byte_len == 0 {
            return false;
        }
        let commands = sdf_atlas_upload_commands(atlas_plan, upload, source_byte_len);
        if !sdf_atlas_upload_batch_is_complete(pages, upload, &commands) {
            return false;
        }
        let mut prepared_uploads = WgpuTextureUploadBatch::new();
        let mut page_index = 0_usize;
        for mut command in commands {
            let texture = match command.page_key.format {
                GlyphAtlasFormat::Sdf => &self.sdf_texture,
                GlyphAtlasFormat::Msdf => &self.msdf_texture,
                _ => return false,
            };
            while pages
                .get(page_index)
                .is_some_and(|page| page.page_key < command.page_key)
            {
                page_index = page_index.saturating_add(1);
            }
            let Some(page) = pages.get(page_index) else {
                return false;
            };
            if page.page_key != command.page_key {
                return false;
            }
            let Ok(page_source_offset) = u64::try_from(page.source_offset) else {
                return false;
            };
            let Some(relative_source_offset) =
                command.source_offset.checked_sub(page_source_offset)
            else {
                return false;
            };
            command.source_offset = relative_source_offset;
            let write = glyph_atlas_texture_upload_write(command);
            let Some(source_range) =
                glyph_atlas_texture_upload_source_range(write, command.upload_byte_len)
            else {
                return false;
            };
            let Some(texture_upload) = WgpuTextureUpload::new(
                texture.clone(),
                glyph_atlas_texture_upload_region(write),
                write.bytes_per_row,
                write.rows_per_image,
                page.pixels.clone(),
                source_range,
            ) else {
                return false;
            };
            prepared_uploads.push(texture_upload);
        }
        texture_uploads.append(prepared_uploads);
        true
    }
}

fn sdf_atlas_upload_batch_is_complete(
    pages: &[SdfAtlasBakePage],
    upload: &SdfAtlasUploadReport,
    commands: &[super::super::sdf_upload::SdfAtlasUploadCommand],
) -> bool {
    if upload.mode == super::super::sdf_upload::SdfAtlasUploadMode::None {
        return upload.byte_len == 0 && upload.dirty_pages.is_empty() && commands.is_empty();
    }
    if commands.len() != upload.dirty_pages.len() {
        return false;
    }
    if !pages
        .windows(2)
        .all(|pair| pair[0].page_key < pair[1].page_key)
        || !commands
            .windows(2)
            .all(|pair| pair[0].page_key < pair[1].page_key)
    {
        return false;
    }

    let mut expected_source_offset = 0_usize;
    for page in pages {
        if page.source_offset != expected_source_offset || page.byte_len != page.pixels.len() {
            return false;
        }
        let Some(next_source_offset) = expected_source_offset.checked_add(page.byte_len) else {
            return false;
        };
        expected_source_offset = next_source_offset;
    }

    let mut command_byte_len = 0_usize;
    let mut page_index = 0_usize;
    for (command, dirty_page) in commands.iter().zip(&upload.dirty_pages) {
        while pages
            .get(page_index)
            .is_some_and(|page| page.page_key < command.page_key)
        {
            page_index = page_index.saturating_add(1);
        }
        if command.page_key != dirty_page.page_key
            || command.upload_byte_len != dirty_page.byte_len
            || pages.get(page_index).map(|page| page.page_key) != Some(command.page_key)
        {
            return false;
        }
        let Some(next_byte_len) = command_byte_len.checked_add(command.upload_byte_len) else {
            return false;
        };
        command_byte_len = next_byte_len;
    }
    command_byte_len == upload.byte_len
}

#[cfg(test)]
mod tests {
    use super::super::super::sdf_upload::{
        SdfAtlasUploadMode, SdfAtlasUploadPageReport, SdfAtlasUploadReport,
    };
    use super::*;
    use crate::text::atlas::{
        GlyphAtlasPageKey, GlyphAtlasRect, GlyphAtlasSamplingSemantics, GlyphAtlasUploadCommand,
        GlyphAtlasUploadMode,
    };

    #[test]
    fn sdf_atlas_upload_batch_requires_one_complete_command_per_dirty_page() {
        let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
        let pages = [sdf_bake_page(page_key, 16)];
        let upload = sdf_upload_report(page_key, 16);

        assert!(!sdf_atlas_upload_batch_is_complete(&pages, &upload, &[]));
        assert!(sdf_atlas_upload_batch_is_complete(
            &pages,
            &upload,
            &[sdf_upload_command(page_key, 16)],
        ));
    }

    #[test]
    fn sdf_atlas_upload_batch_rejects_page_metadata_that_exceeds_its_payload() {
        let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
        let pages = [SdfAtlasBakePage {
            page_key,
            source_offset: 0,
            byte_len: 16,
            pixels: vec![0_u8; 8].into(),
        }];
        let upload = sdf_upload_report(page_key, 16);

        assert!(!sdf_atlas_upload_batch_is_complete(
            &pages,
            &upload,
            &[sdf_upload_command(page_key, 16)],
        ));
    }

    #[test]
    fn sdf_atlas_upload_batch_rejects_dirty_pages_in_none_mode() {
        let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
        let pages = [sdf_bake_page(page_key, 16)];
        let upload = SdfAtlasUploadReport {
            mode: SdfAtlasUploadMode::None,
            byte_len: 0,
            dirty_pages: vec![SdfAtlasUploadPageReport {
                page_key,
                dirty_rect: crate::text::sdf::SdfAtlasRect {
                    x: 0,
                    y: 0,
                    width: 16,
                    height: 1,
                },
                byte_len: 16,
            }],
            ..Default::default()
        };

        assert!(!sdf_atlas_upload_batch_is_complete(&pages, &upload, &[]));
    }

    fn sdf_bake_page(page_key: GlyphAtlasPageKey, byte_len: usize) -> SdfAtlasBakePage {
        SdfAtlasBakePage {
            page_key,
            source_offset: 0,
            byte_len,
            pixels: vec![0_u8; byte_len].into(),
        }
    }

    fn sdf_upload_report(page_key: GlyphAtlasPageKey, byte_len: usize) -> SdfAtlasUploadReport {
        SdfAtlasUploadReport {
            mode: SdfAtlasUploadMode::FullTexture,
            byte_len,
            full_texture: true,
            dirty_slot_count: 1,
            dirty_rect: Some(crate::text::sdf::SdfAtlasRect {
                x: 0,
                y: 0,
                width: byte_len as u32,
                height: 1,
            }),
            dirty_byte_len: byte_len,
            dirty_pages: vec![SdfAtlasUploadPageReport {
                page_key,
                dirty_rect: crate::text::sdf::SdfAtlasRect {
                    x: 0,
                    y: 0,
                    width: byte_len as u32,
                    height: 1,
                },
                byte_len,
            }],
        }
    }

    fn sdf_upload_command(
        page_key: GlyphAtlasPageKey,
        upload_byte_len: usize,
    ) -> GlyphAtlasUploadCommand {
        GlyphAtlasUploadCommand {
            mode: GlyphAtlasUploadMode::FullPage,
            page_key,
            page_generation: 0,
            sampling_semantics: GlyphAtlasSamplingSemantics::SignedDistance,
            rect: GlyphAtlasRect {
                x: 0,
                y: 0,
                width: upload_byte_len as u32,
                height: 1,
            },
            source_offset: 0,
            bytes_per_row: upload_byte_len as u32,
            rows_per_image: 1,
            upload_byte_len,
        }
    }
}
