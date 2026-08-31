use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasRect,
    GlyphAtlasSamplingSemantics, GlyphAtlasSet, GlyphAtlasUploadMode,
};
use super::*;
use crate::core::math::UVec2;

mod dirty_upload;
mod draw_batches;
mod persistent_slots;
mod retry;

#[test]
fn render_text_atlas_bitmap_run_allocates_bitmap_formats_to_distinct_pages() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 22.0, 96),
            source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 36.0, 64),
        ],
        UVec2::new(32, 32),
        7,
        1,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.atlas.page_count(), 3);
    assert_eq!(plan.glyphs.len(), 3);
    assert_eq!(plan.draw_glyphs.len(), 3);
    assert_eq!(plan.dirty_pages.len(), 3);
    assert_eq!(plan.upload_commands.len(), 3);
    assert!(plan.atlas.page(GlyphAtlasFormat::AlphaMask, 0).is_some());
    assert!(plan.atlas.page(GlyphAtlasFormat::SubpixelMask, 0).is_some());
    assert!(plan.atlas.page(GlyphAtlasFormat::Color, 0).is_some());

    assert_eq!(
        plan.glyphs[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(
        plan.glyphs[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0)
    );
    assert_eq!(
        plan.glyphs[2].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0)
    );
    assert!(
        plan.glyphs
            .iter()
            .all(|glyph| glyph.atlas_rect.x == 0 && glyph.atlas_rect.y == 0)
    );
}

#[test]
fn render_text_atlas_bitmap_run_reserves_new_page_on_shelf_overflow() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 4.0, 144),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 20.0, 144),
        ],
        UVec2::new(16, 16),
        11,
        2,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.atlas.page_count(), 2);
    assert_eq!(
        plan.glyphs[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(
        plan.glyphs[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 1)
    );
    assert_eq!(plan.dirty_pages.len(), 2);
    assert_eq!(
        plan.dirty_pages[0].merged_rect(),
        Some(atlas_rect(0, 0, 12, 12))
    );
    assert_eq!(
        plan.dirty_pages[1].merged_rect(),
        Some(atlas_rect(0, 0, 12, 12))
    );
}

#[test]
fn render_text_atlas_bitmap_run_rebuilds_persistent_page_and_invalidates_slots() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(page_key, UVec2::new(16, 16)).with_generation(2),
    );

    let plan = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        atlas,
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(12, 12),
            4.0,
            144,
        )],
        UVec2::new(16, 16),
        12,
        1,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.atlas.page_count(), 1);
    assert_eq!(
        plan.atlas
            .page(GlyphAtlasFormat::AlphaMask, 0)
            .map(|page| page.generation),
        Some(3)
    );
    assert_eq!(plan.rebuilt_pages, vec![page_key]);
    assert_eq!(
        plan.slot_invalidations,
        vec![GlyphAtlasBitmapSlotInvalidation {
            page_key,
            page_generation: 3,
        }]
    );
    assert_eq!(plan.upload_commands.len(), 1);
    assert_eq!(plan.upload_commands[0].mode, GlyphAtlasUploadMode::FullPage);
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 16, 16));
}

#[test]
fn render_text_atlas_bitmap_run_emits_upload_commands_for_dirty_pages() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 22.0, 96),
        ],
        UVec2::new(32, 32),
        19,
        1,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.upload_commands.len(), 2);
    assert_eq!(
        plan.upload_commands[0].mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(
        plan.upload_commands[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 8, 4));
    assert_eq!(plan.upload_commands[0].bytes_per_row, 32);
    assert_eq!(plan.upload_commands[0].upload_byte_len, 32);

    assert_eq!(
        plan.upload_commands[1].mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(
        plan.upload_commands[1].sampling_semantics,
        GlyphAtlasSamplingSemantics::SubpixelCoverage
    );
    assert_eq!(plan.upload_commands[1].bytes_per_row, 32 * 4);
    assert_eq!(plan.upload_commands[1].upload_byte_len, 6 * 4 * 4);
}

#[test]
fn render_text_atlas_bitmap_run_emits_upload_copies_for_staging_sources() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 22.0, 96),
        ],
        UVec2::new(32, 32),
        19,
        1,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(
        plan.upload_copies,
        vec![
            GlyphAtlasBitmapUploadCopy {
                source_index: 0,
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
                atlas_rect: atlas_rect(0, 0, 8, 4),
                content_size: UVec2::new(8, 4),
                source_bytes_per_row: 8,
                source_byte_len: 32,
            },
            GlyphAtlasBitmapUploadCopy {
                source_index: 1,
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
                atlas_rect: atlas_rect(0, 0, 6, 4),
                content_size: UVec2::new(6, 4),
                source_bytes_per_row: 6 * 4,
                source_byte_len: 96,
            },
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_upload_staging_plan_copies_sources_into_page_rows() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 8.0, 8),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 16.0, 8),
        ],
        UVec2::new(8, 4),
        21,
        1,
        0,
    );
    let first_bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let second_bytes = [101, 102, 103, 104, 105, 106, 107, 108];

    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [
            GlyphAtlasBitmapUploadSourceBytes::new(0, &first_bytes),
            GlyphAtlasBitmapUploadSourceBytes::new(1, &second_bytes),
        ],
    );

    assert!(!staging.has_failures());
    assert_eq!(staging.pages.len(), 1);
    assert_eq!(
        staging.pages[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(staging.pages[0].target_rect, atlas_rect(0, 0, 8, 2));
    assert_eq!(staging.pages[0].bytes_per_row, 8);
    assert_eq!(
        staging.pages[0].bytes,
        vec![
            1, 2, 3, 4, 101, 102, 103, 104, 5, 6, 7, 8, 105, 106, 107, 108,
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_upload_staging_plan_uses_packed_rgba_stride() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::SubpixelMask,
            UVec2::new(2, 1),
            8.0,
            8,
        )],
        UVec2::new(4, 2),
        22,
        1,
        0,
    );
    let source_bytes = [10, 20, 30, 40, 50, 60, 70, 80];

    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &source_bytes)],
    );

    assert!(!staging.has_failures());
    assert_eq!(staging.pages.len(), 1);
    assert_eq!(staging.pages[0].target_rect, atlas_rect(0, 0, 2, 1));
    assert_eq!(staging.pages[0].bytes_per_row, 8);
    assert_eq!(
        staging.pages[0].bytes,
        vec![10, 20, 30, 40, 50, 60, 70, 80,]
    );
}

#[test]
fn render_text_atlas_bitmap_staged_upload_plan_binds_page_bytes_to_upload_command() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 8.0, 8),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 16.0, 8),
        ],
        UVec2::new(8, 4),
        23,
        1,
        0,
    );
    let first_bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let second_bytes = [101, 102, 103, 104, 105, 106, 107, 108];
    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [
            GlyphAtlasBitmapUploadSourceBytes::new(0, &first_bytes),
            GlyphAtlasBitmapUploadSourceBytes::new(1, &second_bytes),
        ],
    );

    let staged = glyph_atlas_bitmap_staged_upload_plan(&staging, &plan.upload_commands);

    assert!(!staged.has_failures());
    assert_eq!(staged.uploads.len(), 1);
    assert_eq!(staged.uploads[0].staging_page_index, 0);
    assert_eq!(staged.uploads[0].staging_page_byte_len, 16);
    assert_eq!(
        staged.uploads[0].command.mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(staged.uploads[0].command.rect, atlas_rect(0, 0, 8, 2));
    assert_eq!(staged.uploads[0].command.source_offset, 0);
    assert_eq!(staged.uploads[0].command.bytes_per_row, 8);
    assert_eq!(staged.uploads[0].command.rows_per_image, 2);
    assert_eq!(staged.uploads[0].command.upload_byte_len, 16);
}

#[test]
fn render_text_atlas_bitmap_staged_upload_plan_reports_missing_staging_page() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(4, 2),
            8.0,
            8,
        )],
        UVec2::new(8, 4),
        24,
        1,
        0,
    );
    let staged = glyph_atlas_bitmap_staged_upload_plan(
        &GlyphAtlasBitmapUploadStagingPlan::default(),
        &plan.upload_commands,
    );

    assert!(staged.has_failures());
    assert!(staged.uploads.is_empty());
    assert_eq!(
        staged.failures,
        vec![GlyphAtlasBitmapStagedUploadFailure {
            upload_command_index: 0,
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            reason: GlyphAtlasBitmapStagedUploadFailureReason::MissingStagingPage,
        }]
    );
}

#[test]
fn render_text_atlas_bitmap_staged_upload_plan_reports_short_page_bytes() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(4, 2),
            8.0,
            8,
        )],
        UVec2::new(8, 4),
        25,
        1,
        0,
    );
    let staging = GlyphAtlasBitmapUploadStagingPlan {
        pages: vec![GlyphAtlasBitmapPageUploadStaging {
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            page_generation: 0,
            target_rect: atlas_rect(0, 0, 4, 2),
            bytes_per_row: 8,
            bytes: vec![0; 4].into(),
        }],
        failures: Vec::new(),
    };

    let staged = glyph_atlas_bitmap_staged_upload_plan(&staging, &plan.upload_commands);

    assert!(staged.has_failures());
    assert!(staged.uploads.is_empty());
    assert_eq!(
        staged.failures,
        vec![GlyphAtlasBitmapStagedUploadFailure {
            upload_command_index: 0,
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            reason: GlyphAtlasBitmapStagedUploadFailureReason::SourceRangeOutOfBounds,
        }]
    );
}

#[test]
fn render_text_atlas_bitmap_prepared_upload_plan_builds_staging_and_uploads() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 8.0, 8),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 16.0, 8),
        ],
        UVec2::new(8, 4),
        26,
        1,
        0,
    );
    let first_bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let second_bytes = [101, 102, 103, 104, 105, 106, 107, 108];

    let prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &plan,
        [
            GlyphAtlasBitmapUploadSourceBytes::new(0, &first_bytes),
            GlyphAtlasBitmapUploadSourceBytes::new(1, &second_bytes),
        ],
    );

    assert!(!prepared.has_failures());
    assert_eq!(prepared.staging.pages.len(), 1);
    assert_eq!(prepared.staged_uploads.uploads.len(), 1);
    assert_eq!(prepared.staged_uploads.uploads[0].staging_page_index, 0);
    assert_eq!(prepared.staged_uploads.uploads[0].staging_page_byte_len, 16);
}

#[test]
fn render_text_atlas_bitmap_texture_upload_request_plan_projects_renderer_write_fields() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 8.0, 8),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 16.0, 8),
        ],
        UVec2::new(8, 4),
        27,
        1,
        0,
    );
    let first_bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let second_bytes = [101, 102, 103, 104, 105, 106, 107, 108];
    let prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &plan,
        [
            GlyphAtlasBitmapUploadSourceBytes::new(0, &first_bytes),
            GlyphAtlasBitmapUploadSourceBytes::new(1, &second_bytes),
        ],
    );

    let request_plan = glyph_atlas_bitmap_texture_upload_request_plan(&prepared.staged_uploads);

    assert!(request_plan.has_requests());
    assert_eq!(request_plan.skipped_failure_count, 0);
    assert_eq!(
        request_plan.requests,
        vec![GlyphAtlasBitmapTextureUploadRequest {
            staging_page_index: 0,
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            page_generation: 0,
            origin_xy: UVec2::new(0, 0),
            origin_layer: 0,
            extent: UVec2::new(8, 2),
            source_offset: 0,
            bytes_per_row: 8,
            rows_per_image: 2,
            upload_byte_len: 16,
            staging_page_byte_len: 16,
        }]
    );
}

#[test]
fn render_text_atlas_bitmap_texture_upload_request_plan_skips_failed_staged_uploads() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(4, 2),
            8.0,
            8,
        )],
        UVec2::new(8, 4),
        28,
        1,
        0,
    );
    let staged = glyph_atlas_bitmap_staged_upload_plan(
        &GlyphAtlasBitmapUploadStagingPlan::default(),
        &plan.upload_commands,
    );

    let request_plan = glyph_atlas_bitmap_texture_upload_request_plan(&staged);

    assert!(!request_plan.has_requests());
    assert_eq!(request_plan.requests.len(), 0);
    assert_eq!(request_plan.skipped_failure_count, 1);
}

#[test]
fn render_text_atlas_bitmap_texture_upload_request_plan_requeues_stale_page_generation() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(4, 2),
            8.0,
            8,
        )],
        UVec2::new(8, 4),
        30,
        1,
        0,
    );
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &bytes)],
    );
    let stale_atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            UVec2::new(8, 4),
        )
        .with_generation(1),
    );

    let request_plan = glyph_atlas_bitmap_texture_upload_request_plan_with_atlas(
        &prepared.staged_uploads,
        &stale_atlas,
    );

    assert!(request_plan.requests.is_empty());
    assert_eq!(request_plan.stale_page_generation_count, 1);
    assert_eq!(
        request_plan.requeued_uploads,
        vec![GlyphAtlasBitmapRequeuedUpload {
            upload_command_index: 0,
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            requested_page_generation: 0,
            current_page_generation: Some(1),
            reason: GlyphAtlasBitmapRequeueReason::PageGenerationMismatch,
        }]
    );
}

#[test]
fn render_text_atlas_bitmap_texture_upload_request_plan_requeues_invalid_face_uploads() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(4, 2),
            8.0,
            8,
        )],
        UVec2::new(8, 4),
        31,
        1,
        0,
    );
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &bytes)],
    );

    let request_plan = glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity(
        &prepared.staged_uploads,
        &plan.atlas,
        GlyphAtlasBitmapFaceValidity::Invalidated,
    );

    assert!(request_plan.requests.is_empty());
    assert_eq!(request_plan.face_invalidated_count, 1);
    assert_eq!(
        request_plan.requeued_uploads,
        vec![GlyphAtlasBitmapRequeuedUpload {
            upload_command_index: 0,
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            requested_page_generation: 0,
            current_page_generation: Some(0),
            reason: GlyphAtlasBitmapRequeueReason::FaceInvalidated,
        }]
    );
}

#[test]
fn render_text_atlas_bitmap_prepared_upload_plan_skips_uploads_when_staging_fails() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 8.0, 8),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 16.0, 8),
        ],
        UVec2::new(8, 4),
        29,
        1,
        0,
    );
    let short_bytes = [1, 2, 3];

    let prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &short_bytes)],
    );

    assert!(prepared.has_failures());
    assert!(prepared.staging.has_failures());
    assert!(prepared.staged_uploads.uploads.is_empty());
    assert!(prepared.staged_uploads.failures.is_empty());
}

#[test]
fn render_text_atlas_bitmap_upload_staging_plan_reports_missing_or_mismatched_sources() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 8.0, 8),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 2), 16.0, 8),
        ],
        UVec2::new(8, 4),
        30,
        1,
        0,
    );
    let short_bytes = [1, 2, 3];

    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &short_bytes)],
    );

    assert!(staging.has_failures());
    assert_eq!(staging.pages.len(), 0);
    assert_eq!(
        staging.failures,
        vec![
            GlyphAtlasBitmapUploadStagingFailure {
                source_index: 0,
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
                reason: GlyphAtlasBitmapUploadStagingFailureReason::SourceLengthMismatch {
                    expected: 8,
                    actual: 3,
                },
            },
            GlyphAtlasBitmapUploadStagingFailure {
                source_index: 1,
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
                reason: GlyphAtlasBitmapUploadStagingFailureReason::MissingSourceBytes,
            },
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_run_records_failures_for_invalid_sources() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 4), 4.0, 16),
            source(GlyphAtlasFormat::Sdf, UVec2::new(4, 4), 12.0, 16),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(0, 4), 20.0, 0),
            source(GlyphAtlasFormat::Color, UVec2::new(2, 2), 28.0, 15),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(20, 20), 36.0, 400),
        ],
        UVec2::new(16, 16),
        13,
        0,
        2,
    );

    assert!(plan.glyphs.is_empty());
    assert_eq!(plan.atlas.page_count(), 0);
    assert_eq!(plan.blocked_glyphs.len(), 1);
    assert_eq!(plan.blocked_glyphs[0].source_index, 0);
    assert_eq!(
        plan.blocked_glyphs[0].source.format,
        GlyphAtlasFormat::AlphaMask
    );
    assert_eq!(plan.blocked_glyphs[0].retry_frame_index, 14);
    assert_eq!(plan.placeholder_glyphs.len(), 1);
    assert_eq!(plan.placeholder_glyphs[0].source_index, 0);
    assert_eq!(
        plan.placeholder_glyphs[0].mode,
        GlyphAtlasBitmapPlaceholderMode::TransparentQuad
    );
    assert_eq!(plan.placeholder_glyphs[0].retry_frame_index, 14);
    assert_eq!(
        plan.placeholder_glyphs[0].screen_rect,
        GlyphAtlasScreenRect::new(4.0, 6.0, 4.0, 4.0)
    );
    assert_eq!(
        plan.allocation_failures,
        vec![
            failure(
                0,
                GlyphAtlasFormat::AlphaMask,
                GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked
            ),
            failure(
                1,
                GlyphAtlasFormat::Sdf,
                GlyphAtlasBitmapAllocationFailureReason::UnsupportedFormat
            ),
            failure(
                2,
                GlyphAtlasFormat::AlphaMask,
                GlyphAtlasBitmapAllocationFailureReason::EmptyContent
            ),
            failure(
                3,
                GlyphAtlasFormat::Color,
                GlyphAtlasBitmapAllocationFailureReason::DataLengthMismatch {
                    expected: 16,
                    actual: 15,
                }
            ),
            failure(
                4,
                GlyphAtlasFormat::AlphaMask,
                GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph
            ),
        ]
    );
}

#[test]
fn bitmap_upload_staging_does_not_allocate_per_glyph_row_range_vec() {
    let source = include_str!("staging.rs");

    assert!(!source.contains("let mut row_ranges = Vec::with_capacity"));
}

fn source(
    format: GlyphAtlasFormat,
    content_size: UVec2,
    x: f32,
    source_byte_len: usize,
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        raster_key: None,
        format,
        content_size,
        screen_rect: GlyphAtlasScreenRect::new(
            x,
            6.0,
            content_size.x as f32,
            content_size.y as f32,
        ),
        foreground_color: [0.86, 0.88, 0.9, 1.0],
        background_color: [0.08, 0.09, 0.1, 1.0],
        source_byte_len,
    }
}

fn queued_glyph(
    source_index: usize,
    source: GlyphAtlasBitmapSource,
    retry_frame_index: u64,
) -> GlyphAtlasBitmapQueuedGlyph {
    GlyphAtlasBitmapQueuedGlyph {
        source_index,
        source,
        retry_frame_index,
    }
}

fn failure(
    source_index: usize,
    format: GlyphAtlasFormat,
    reason: GlyphAtlasBitmapAllocationFailureReason,
) -> GlyphAtlasBitmapAllocationFailure {
    GlyphAtlasBitmapAllocationFailure {
        source_index,
        format,
        reason,
    }
}

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
