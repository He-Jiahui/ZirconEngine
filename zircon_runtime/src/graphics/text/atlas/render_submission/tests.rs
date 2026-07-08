use super::super::render_contract::GlyphAtlasBlendMode;
use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GlyphAtlasBitmapAllocationFailureReason, GlyphAtlasBitmapPlaceholderGlyph,
    GlyphAtlasBitmapPlaceholderMode, GlyphAtlasBitmapQueuedGlyph,
    GlyphAtlasBitmapRetryBackpressurePolicy, GlyphAtlasBitmapRetrySourceOrigin,
    GlyphAtlasBitmapSource, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat, GlyphAtlasPageKey,
    GlyphAtlasPageSpec, GlyphAtlasRect, GlyphAtlasSet, GlyphAtlasUploadMode,
};
use super::*;
use crate::core::math::UVec2;

mod retry_frame;

#[test]
fn render_text_atlas_bitmap_submission_carries_uploads_batches_and_gpu_commands() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96),
        ],
        UVec2::new(32, 32),
        31,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.run.glyphs.len(), 2);
    assert_eq!(plan.upload_commands().len(), 2);
    assert_eq!(
        plan.upload_commands()[0].mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(
        plan.upload_commands()[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(plan.draw_batches.visible_glyph_count, 2);
    assert_eq!(plan.draw_batches.skipped_glyph_count, 0);
    assert_eq!(plan.draw_batches.batches.len(), 2);
    assert_eq!(plan.gpu_draw.vertex_count(), 12);
    assert_eq!(plan.gpu_draw.draw_commands.len(), 2);
    assert!(plan.gpu_draw.requires_background_composite);
    assert_eq!(
        plan.gpu_draw.draw_commands[1].render_contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
}

#[test]
fn render_text_atlas_bitmap_submission_culls_draws_without_losing_uploads() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 4),
            96.0,
            32,
        )],
        UVec2::new(32, 32),
        37,
        1,
        2,
        UVec2::new(48, 24),
        GlyphAtlasScreenRect::new(0.0, 0.0, 48.0, 24.0),
    );

    assert_eq!(plan.run.glyphs.len(), 1);
    assert_eq!(plan.upload_commands().len(), 1);
    assert_eq!(plan.draw_batches.visible_glyph_count, 0);
    assert_eq!(plan.draw_batches.skipped_glyph_count, 1);
    assert_eq!(plan.draw_batches.batches.len(), 0);
    assert_eq!(plan.gpu_draw.vertex_count(), 0);
    assert_eq!(plan.gpu_draw.draw_commands.len(), 0);
}

#[test]
fn render_text_atlas_bitmap_submission_preserves_failures_without_gpu_work() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::Sdf, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(40, 40), 24.0, 1600),
        ],
        UVec2::new(32, 32),
        41,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.run.glyphs.len(), 0);
    assert_eq!(plan.upload_commands().len(), 0);
    assert_eq!(plan.allocation_failure_count(), 2);
    assert_eq!(
        plan.run.allocation_failures[0].reason,
        GlyphAtlasBitmapAllocationFailureReason::UnsupportedFormat
    );
    assert_eq!(
        plan.run.allocation_failures[1].reason,
        GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph
    );
    assert_eq!(plan.draw_batches.visible_glyph_count, 0);
    assert_eq!(plan.gpu_draw.vertex_count(), 0);
}

#[test]
fn render_text_atlas_bitmap_submission_blocks_same_frame_eviction() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 4.0, 144),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 20.0, 144),
        ],
        UVec2::new(16, 16),
        43,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.rebuilt_page_count(), 0);
    assert_eq!(plan.run.glyphs.len(), 1);
    assert_eq!(plan.allocation_failure_count(), 1);
    assert_eq!(plan.run.blocked_glyphs.len(), 1);
    assert_eq!(plan.run.blocked_glyphs[0].source_index, 1);
    assert_eq!(plan.run.blocked_glyphs[0].retry_frame_index, 44);
    assert_eq!(plan.run.placeholder_glyphs.len(), 1);
    assert_eq!(plan.run.placeholder_glyphs[0].source_index, 1);
    assert_eq!(plan.run.placeholder_glyphs[0].retry_frame_index, 44);
    assert_eq!(plan.placeholder_draws.visible_placeholder_count, 1);
    assert_eq!(plan.placeholder_draws.skipped_placeholder_count, 0);
    assert_eq!(plan.placeholder_draws.draws[0].source_index, 1);
    assert_eq!(
        plan.placeholder_draws.draws[0].screen_rect,
        GlyphAtlasScreenRect::new(20.0, 6.0, 12.0, 12.0)
    );
    assert_eq!(
        plan.run.allocation_failures[0].reason,
        GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked
    );
    assert_eq!(plan.upload_commands().len(), 1);
    assert_eq!(
        plan.upload_commands()[0].mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(plan.upload_commands()[0].rect, atlas_rect(0, 0, 12, 12));
    assert_eq!(plan.gpu_draw.draw_commands.len(), 1);
}

#[test]
fn render_text_atlas_bitmap_submission_clips_placeholder_draw_plan() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 4.0, 144),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 96.0, 144),
        ],
        UVec2::new(16, 16),
        44,
        1,
        2,
        UVec2::new(48, 24),
        GlyphAtlasScreenRect::new(0.0, 0.0, 48.0, 24.0),
    );

    let report = plan.submission_report();

    assert_eq!(plan.run.placeholder_glyphs.len(), 1);
    assert_eq!(plan.placeholder_draws.visible_placeholder_count, 0);
    assert_eq!(plan.placeholder_draws.skipped_placeholder_count, 1);
    assert_eq!(plan.placeholder_draws.draws.len(), 0);
    assert_eq!(report.placeholder_glyph_count, 1);
    assert_eq!(report.visible_placeholder_count, 0);
    assert_eq!(report.skipped_placeholder_count, 1);
    assert!(!report.has_placeholder_work());
}

#[test]
fn render_text_atlas_bitmap_submission_appends_worker_pending_placeholders() {
    let mut plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 4),
            8.0,
            32,
        )],
        UVec2::new(32, 32),
        45,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    plan.append_placeholder_glyphs(
        [
            worker_pending_placeholder(1, 48.0, 46),
            worker_pending_placeholder(2, 120.0, 46),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );
    let report = plan.submission_report();

    assert_eq!(plan.run.glyphs.len(), 1);
    assert_eq!(plan.run.placeholder_glyphs.len(), 2);
    assert_eq!(plan.placeholder_draws.visible_placeholder_count, 1);
    assert_eq!(plan.placeholder_draws.skipped_placeholder_count, 1);
    assert_eq!(plan.placeholder_draws.draws[0].source_index, 1);
    assert_eq!(report.placeholder_glyph_count, 2);
    assert_eq!(report.visible_placeholder_count, 1);
    assert_eq!(report.skipped_placeholder_count, 1);
    assert!(report.has_placeholder_work());
}

#[test]
fn render_text_atlas_bitmap_submission_report_summarizes_upload_and_gpu_work() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96),
        ],
        UVec2::new(32, 32),
        47,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.submission_report();

    assert_eq!(report.source_count, 2);
    assert_eq!(report.allocated_glyph_count, 2);
    assert_eq!(report.visible_glyph_count, 2);
    assert_eq!(report.skipped_glyph_count, 0);
    assert_eq!(report.allocation_failure_count, 0);
    assert_eq!(report.blocked_retry_count, 0);
    assert_eq!(report.placeholder_glyph_count, 0);
    assert_eq!(report.visible_placeholder_count, 0);
    assert_eq!(report.skipped_placeholder_count, 0);
    assert_eq!(report.next_retry_frame_index, None);
    assert_eq!(report.source_validation_failure_count(), 0);
    assert_eq!(report.atlas_capacity_failure_count(), 0);
    assert_eq!(report.dirty_page_count, 2);
    assert_eq!(report.rebuilt_page_count, 0);
    assert_eq!(report.upload_command_count, 2);
    assert_eq!(report.full_page_upload_count, 0);
    assert_eq!(report.partial_upload_count, 2);
    assert_eq!(report.upload_byte_len, 128);
    assert_eq!(report.upload_copy_count, 2);
    assert_eq!(report.upload_copy_byte_len, 128);
    assert_eq!(report.draw_batch_count, 2);
    assert_eq!(report.pipeline_count, 2);
    assert_eq!(report.gpu_batch_count, 2);
    assert_eq!(report.draw_command_count, 2);
    assert_eq!(report.vertex_count, 12);
    assert!(report.requires_background_composite);
    assert!(report.has_upload_work());
    assert!(report.has_upload_copy_work());
    assert!(report.has_gpu_work());
    assert!(!report.has_failures());
    assert!(!report.has_placeholder_work());
}

#[test]
fn render_text_atlas_bitmap_submission_report_keeps_upload_only_culled_glyphs() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 4),
            96.0,
            32,
        )],
        UVec2::new(32, 32),
        53,
        1,
        2,
        UVec2::new(48, 24),
        GlyphAtlasScreenRect::new(0.0, 0.0, 48.0, 24.0),
    );

    let report = glyph_atlas_bitmap_render_submission_report(&plan);

    assert_eq!(report.source_count, 1);
    assert_eq!(report.allocated_glyph_count, 1);
    assert_eq!(report.visible_glyph_count, 0);
    assert_eq!(report.skipped_glyph_count, 1);
    assert_eq!(report.upload_command_count, 1);
    assert_eq!(report.partial_upload_count, 1);
    assert_eq!(report.upload_byte_len, 32);
    assert_eq!(report.upload_copy_count, 1);
    assert_eq!(report.upload_copy_byte_len, 32);
    assert_eq!(report.draw_batch_count, 0);
    assert_eq!(report.draw_command_count, 0);
    assert!(report.has_upload_work());
    assert!(report.has_upload_copy_work());
    assert!(!report.has_gpu_work());
}

#[test]
fn render_text_atlas_bitmap_submission_report_counts_full_page_uploads() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(16, 16),
            4.0,
            256,
        )],
        UVec2::new(16, 16),
        59,
        1,
        0,
        UVec2::new(48, 24),
        GlyphAtlasScreenRect::new(0.0, 0.0, 48.0, 24.0),
    );

    let report = plan.submission_report();

    assert_eq!(report.source_count, 1);
    assert_eq!(report.upload_command_count, 1);
    assert_eq!(report.full_page_upload_count, 1);
    assert_eq!(report.partial_upload_count, 0);
    assert_eq!(report.upload_byte_len, 256);
    assert_eq!(report.upload_copy_count, 1);
    assert_eq!(report.upload_copy_byte_len, 256);
    assert_eq!(report.dirty_page_count, 1);
    assert_eq!(report.visible_glyph_count, 1);
    assert!(report.has_upload_work());
    assert!(report.has_upload_copy_work());
    assert!(report.has_gpu_work());
}

#[test]
fn render_text_atlas_bitmap_submission_report_counts_slot_invalidations() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(page_key, UVec2::new(16, 16)).with_generation(4),
    );
    let plan = glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding(
        atlas,
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(12, 12),
            4.0,
            144,
        )],
        UVec2::new(16, 16),
        60,
        1,
        2,
        UVec2::new(48, 24),
        GlyphAtlasScreenRect::new(0.0, 0.0, 48.0, 24.0),
    );

    let report = plan.submission_report();

    assert_eq!(report.rebuilt_page_count, 1);
    assert_eq!(report.slot_invalidation_count, 1);
    assert_eq!(plan.run.slot_invalidations[0].page_key, page_key);
    assert_eq!(plan.run.slot_invalidations[0].page_generation, 5);
}

#[test]
fn render_text_atlas_bitmap_submission_report_preserves_failure_totals() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::Sdf, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(40, 40), 24.0, 1600),
        ],
        UVec2::new(32, 32),
        61,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.submission_report();

    assert_eq!(report.source_count, 2);
    assert_eq!(report.allocated_glyph_count, 0);
    assert_eq!(report.allocation_failure_count, 2);
    assert_eq!(report.blocked_retry_count, 0);
    assert_eq!(report.placeholder_glyph_count, 0);
    assert_eq!(report.visible_placeholder_count, 0);
    assert_eq!(report.skipped_placeholder_count, 0);
    assert_eq!(report.next_retry_frame_index, None);
    assert_eq!(report.unsupported_format_failure_count, 1);
    assert_eq!(report.empty_content_failure_count, 0);
    assert_eq!(report.data_length_mismatch_failure_count, 0);
    assert_eq!(report.page_reservation_blocked_failure_count, 0);
    assert_eq!(report.oversized_glyph_failure_count, 1);
    assert_eq!(report.source_validation_failure_count(), 1);
    assert_eq!(report.atlas_capacity_failure_count(), 1);
    assert_eq!(report.upload_command_count, 0);
    assert_eq!(report.upload_copy_count, 0);
    assert_eq!(report.upload_copy_byte_len, 0);
    assert_eq!(report.draw_command_count, 0);
    assert_eq!(report.vertex_count, 0);
    assert!(report.has_failures());
    assert!(!report.has_upload_work());
    assert!(!report.has_upload_copy_work());
    assert!(!report.has_gpu_work());
}

#[test]
fn render_text_atlas_bitmap_submission_report_breaks_down_failure_reasons() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(0, 4), 4.0, 0),
            source(GlyphAtlasFormat::Color, UVec2::new(2, 2), 16.0, 15),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 28.0, 144),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 44.0, 144),
        ],
        UVec2::new(16, 16),
        67,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.submission_report();

    assert_eq!(report.source_count, 4);
    assert_eq!(report.allocated_glyph_count, 1);
    assert_eq!(report.allocation_failure_count, 3);
    assert_eq!(report.blocked_retry_count, 1);
    assert_eq!(report.placeholder_glyph_count, 1);
    assert_eq!(report.visible_placeholder_count, 1);
    assert_eq!(report.skipped_placeholder_count, 0);
    assert_eq!(report.next_retry_frame_index, Some(68));
    assert_eq!(report.unsupported_format_failure_count, 0);
    assert_eq!(report.empty_content_failure_count, 1);
    assert_eq!(report.data_length_mismatch_failure_count, 1);
    assert_eq!(report.page_reservation_blocked_failure_count, 1);
    assert_eq!(report.oversized_glyph_failure_count, 0);
    assert_eq!(report.source_validation_failure_count(), 2);
    assert_eq!(report.atlas_capacity_failure_count(), 1);
    assert_eq!(report.upload_command_count, 1);
    assert_eq!(report.partial_upload_count, 1);
    assert_eq!(report.upload_copy_count, 1);
    assert_eq!(report.upload_copy_byte_len, 144);
    assert_eq!(report.draw_command_count, 1);
    assert!(report.has_failures());
    assert!(report.has_upload_work());
    assert!(report.has_upload_copy_work());
    assert!(report.has_gpu_work());
    assert!(report.has_placeholder_work());
}

#[test]
fn render_text_atlas_bitmap_submission_prepares_upload_from_source_bytes() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96),
        ],
        UVec2::new(32, 32),
        71,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );
    let alpha_bytes = vec![7; 32];
    let subpixel_bytes = vec![11; 96];

    let prepared = plan.prepared_upload([
        GlyphAtlasBitmapUploadSourceBytes::new(0, alpha_bytes.as_slice()),
        GlyphAtlasBitmapUploadSourceBytes::new(1, subpixel_bytes.as_slice()),
    ]);

    assert!(!prepared.has_failures());
    assert_eq!(prepared.staging.pages.len(), 2);
    assert_eq!(prepared.staged_uploads.uploads.len(), 2);
    assert_eq!(
        prepared
            .staged_uploads
            .uploads
            .iter()
            .map(|upload| upload.command.upload_byte_len)
            .sum::<usize>(),
        128
    );
    assert!(prepared.staging.pages.iter().any(|page| {
        page.page_key == GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
            && page.bytes_per_row == 32
            && page.bytes.contains(&7)
    }));
    assert!(prepared.staging.pages.iter().any(|page| {
        page.page_key == GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0)
            && page.bytes_per_row == 128
            && page.bytes.contains(&11)
    }));
}

#[test]
fn render_text_atlas_bitmap_submission_prepare_upload_blocks_missing_source_bytes() {
    let plan = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 4),
            8.0,
            32,
        )],
        UVec2::new(32, 32),
        73,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let prepared =
        plan.prepared_upload(std::iter::empty::<GlyphAtlasBitmapUploadSourceBytes<'_>>());

    assert!(prepared.has_failures());
    assert_eq!(prepared.staging.failures.len(), 1);
    assert_eq!(prepared.staged_uploads.uploads.len(), 0);
    assert_eq!(prepared.staged_uploads.failures.len(), 0);
}

fn source(
    format: GlyphAtlasFormat,
    content_size: UVec2,
    x: f32,
    source_byte_len: usize,
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
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

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}

fn worker_pending_placeholder(
    source_index: usize,
    x: f32,
    retry_frame_index: u64,
) -> GlyphAtlasBitmapPlaceholderGlyph {
    GlyphAtlasBitmapPlaceholderGlyph {
        source_index,
        format: GlyphAtlasFormat::AlphaMask,
        screen_rect: GlyphAtlasScreenRect::new(x, 6.0, 8.0, 12.0),
        retry_frame_index,
        mode: GlyphAtlasBitmapPlaceholderMode::TransparentQuad,
    }
}

fn frame_driver_config(
    page_size: UVec2,
    max_pages_per_format: usize,
    padding_px: u32,
) -> GlyphAtlasBitmapRetryFrameDriverConfig {
    GlyphAtlasBitmapRetryFrameDriverConfig {
        page_size,
        max_pages_per_format,
        padding_px,
        backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy::unlimited(),
        viewport_size: UVec2::new(96, 32),
        clip_rect: GlyphAtlasScreenRect::new(0.0, 0.0, 96.0, 32.0),
    }
}
