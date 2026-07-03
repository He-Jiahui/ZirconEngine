use super::super::render_contract::GlyphAtlasBlendMode;
use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GlyphAtlasBitmapAllocationFailureReason, GlyphAtlasBitmapSource, GlyphAtlasFormat,
    GlyphAtlasPageKey, GlyphAtlasRect, GlyphAtlasUploadMode,
};
use super::*;
use crate::core::math::UVec2;

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
    assert_eq!(report.draw_batch_count, 2);
    assert_eq!(report.pipeline_count, 2);
    assert_eq!(report.gpu_batch_count, 2);
    assert_eq!(report.draw_command_count, 2);
    assert_eq!(report.vertex_count, 12);
    assert!(report.requires_background_composite);
    assert!(report.has_upload_work());
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
    assert_eq!(report.draw_batch_count, 0);
    assert_eq!(report.draw_command_count, 0);
    assert!(report.has_upload_work());
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
    assert_eq!(report.dirty_page_count, 1);
    assert_eq!(report.visible_glyph_count, 1);
    assert!(report.has_upload_work());
    assert!(report.has_gpu_work());
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
    assert_eq!(report.draw_command_count, 0);
    assert_eq!(report.vertex_count, 0);
    assert!(report.has_failures());
    assert!(!report.has_upload_work());
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
    assert_eq!(report.draw_command_count, 1);
    assert!(report.has_failures());
    assert!(report.has_upload_work());
    assert!(report.has_gpu_work());
    assert!(report.has_placeholder_work());
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

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
