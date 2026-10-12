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
