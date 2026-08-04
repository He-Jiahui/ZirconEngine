use super::*;
use crate::text::InstancedFaceId;
use crate::text::atlas::{
    GlyphHintingMode, GlyphRasterKey, GlyphSmoothingMode, SyntheticGlyphStyle,
};

fn keyed_source(glyph_id: u32, x: f32) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        raster_key: Some(GlyphRasterKey {
            face: InstancedFaceId(43),
            glyph_id,
            px_size_bucket: 16,
            subpixel_bin: 0,
            vertical_subpixel_bin: 0,
            format: GlyphAtlasFormat::AlphaMask,
            hinting: GlyphHintingMode::Full,
            smoothing: GlyphSmoothingMode::Grayscale,
            synthetic: SyntheticGlyphStyle::default(),
        }),
        ..source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), x, 64)
    }
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_merges_due_retries_and_new_sources() {
    let due_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let deferred_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 80.0, 64);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96);
    let deferred = queued_glyph(11, deferred_source, 93);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_padding(
        [queued_glyph(9, due_source, 91), deferred],
        [new_source],
        UVec2::new(32, 32),
        91,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.frame_input.sources, vec![due_source, new_source]);
    assert_eq!(plan.frame_input.retried_source_count, 1);
    assert_eq!(plan.frame_input.new_source_count, 1);
    assert_eq!(plan.frame_input.deferred_retry_count, 1);
    assert_eq!(plan.submission.run.glyphs.len(), 2);
    assert_eq!(plan.submission.upload_commands().len(), 2);
    assert_eq!(plan.submission.gpu_draw.draw_commands.len(), 2);
    assert_eq!(plan.frame_outcome.next_blocked_glyphs, vec![deferred]);
    assert_eq!(plan.frame_outcome.completed_retried_source_count, 1);
    assert_eq!(plan.frame_outcome.completed_new_source_count, 1);
    assert_eq!(plan.frame_outcome.next_retry_frame_index, Some(93));
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_remaps_blocked_sources_for_next_frame() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let new_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_padding(
        [queued_glyph(14, retry_source, 101)],
        [new_source],
        UVec2::new(32, 32),
        101,
        0,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(
        plan.frame_input.source_origins,
        vec![
            GlyphAtlasBitmapRetrySourceOrigin::Retried {
                source_index: 14,
                retry_frame_index: 101,
            },
            GlyphAtlasBitmapRetrySourceOrigin::New { source_index: 0 },
        ]
    );
    assert_eq!(plan.submission.run.glyphs.len(), 0);
    assert_eq!(plan.submission.run.blocked_glyphs.len(), 2);
    assert_eq!(plan.frame_outcome.next_blocked_glyphs[0].source_index, 14);
    assert_eq!(plan.frame_outcome.next_blocked_glyphs[1].source_index, 0);
    assert_eq!(plan.frame_outcome.blocked_retried_source_count, 1);
    assert_eq!(plan.frame_outcome.blocked_new_source_count, 1);
    assert_eq!(plan.frame_outcome.unmapped_blocked_source_count, 0);
    assert_eq!(plan.frame_outcome.next_retry_frame_index, Some(102));
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_report_combines_submission_and_retry_counts() {
    let due_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let deferred_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 80.0, 64);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_padding(
        [
            queued_glyph(9, due_source, 111),
            queued_glyph(11, deferred_source, 113),
        ],
        [new_source],
        UVec2::new(32, 32),
        111,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.retry_submission_report();

    assert_eq!(report.input_source_count, 2);
    assert_eq!(report.retried_source_count, 1);
    assert_eq!(report.new_source_count, 1);
    assert_eq!(report.deferred_retry_count, 1);
    assert_eq!(report.submission_report.source_count, 2);
    assert_eq!(report.submission_report.allocated_glyph_count, 2);
    assert_eq!(report.submission_report.upload_command_count, 2);
    assert_eq!(report.submission_report.upload_copy_count, 2);
    assert_eq!(report.submission_report.upload_copy_byte_len, 132);
    assert_eq!(report.submission_report.draw_command_count, 2);
    assert_eq!(report.completed_retried_source_count, 1);
    assert_eq!(report.completed_new_source_count, 1);
    assert_eq!(report.blocked_retried_source_count, 0);
    assert_eq!(report.blocked_new_source_count, 0);
    assert_eq!(report.next_blocked_glyph_count, 1);
    assert_eq!(report.next_retry_frame_index, Some(113));
    assert!(report.has_retry_input());
    assert!(report.has_pending_retry_work());
    assert!(!report.has_blocked_retry_work());
    assert!(!report.has_unmapped_blocked_sources());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_report_exposes_blocked_retry_pressure() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let new_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_padding(
        [queued_glyph(14, retry_source, 121)],
        [new_source],
        UVec2::new(32, 32),
        121,
        0,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = glyph_atlas_bitmap_retry_frame_submission_report(&plan);

    assert_eq!(report.input_source_count, 2);
    assert_eq!(report.submission_report.allocated_glyph_count, 0);
    assert_eq!(report.submission_report.allocation_failure_count, 2);
    assert_eq!(report.submission_report.blocked_retry_count, 2);
    assert_eq!(report.submission_report.visible_placeholder_count, 2);
    assert_eq!(report.completed_retried_source_count, 0);
    assert_eq!(report.completed_new_source_count, 0);
    assert_eq!(report.blocked_retried_source_count, 1);
    assert_eq!(report.blocked_new_source_count, 1);
    assert_eq!(report.next_blocked_glyph_count, 2);
    assert_eq!(report.unmapped_blocked_source_count, 0);
    assert_eq!(report.next_retry_frame_index, Some(122));
    assert!(report.has_retry_input());
    assert!(report.has_pending_retry_work());
    assert!(report.has_blocked_retry_work());
    assert!(!report.has_unmapped_blocked_sources());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_report_tracks_backpressured_retries() {
    let first_retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let backpressured_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 40.0, 96);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
        [
            queued_glyph(14, first_retry_source, 131),
            queued_glyph(15, backpressured_source, 131),
        ],
        [new_source],
        UVec2::new(32, 32),
        131,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_sources_per_frame: Some(1),
            max_new_sources_per_frame: None,
            defer_excess_by_frames: 3,
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.retry_submission_report();

    assert_eq!(report.input_source_count, 2);
    assert_eq!(report.retried_source_count, 1);
    assert_eq!(report.new_source_count, 1);
    assert_eq!(report.deferred_retry_count, 1);
    assert_eq!(report.backpressured_retry_count, 1);
    assert_eq!(report.submission_report.source_count, 2);
    assert_eq!(report.submission_report.allocated_glyph_count, 2);
    assert_eq!(report.completed_retried_source_count, 1);
    assert_eq!(report.completed_new_source_count, 1);
    assert_eq!(report.next_blocked_glyph_count, 1);
    assert_eq!(report.next_retry_frame_index, Some(134));
    assert!(report.has_retry_input());
    assert!(report.has_pending_retry_work());
    assert!(report.has_backpressured_retry_work());
    assert!(!report.has_blocked_retry_work());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_report_tracks_backpressured_new_sources() {
    let first_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let deferred_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
        [],
        [first_source, deferred_source],
        UVec2::new(32, 32),
        241,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_sources_per_frame: None,
            max_new_sources_per_frame: Some(1),
            defer_excess_by_frames: 4,
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.retry_submission_report();

    assert_eq!(report.input_source_count, 1);
    assert_eq!(report.new_source_count, 1);
    assert_eq!(report.deferred_new_source_count, 1);
    assert_eq!(report.backpressured_new_source_count, 1);
    assert_eq!(report.submission_report.source_count, 1);
    assert_eq!(report.submission_report.allocated_glyph_count, 1);
    assert_eq!(report.completed_new_source_count, 1);
    assert_eq!(report.next_blocked_glyph_count, 1);
    assert_eq!(report.next_retry_frame_index, Some(245));
    assert!(report.has_pending_retry_work());
    assert!(report.has_backpressured_new_work());
    assert!(!report.has_backpressured_retry_work());
    assert_eq!(
        plan.frame_outcome.next_blocked_glyphs,
        vec![queued_glyph(1, deferred_source, 245)]
    );
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_report_exposes_byte_budget_pressure() {
    let first_retry = source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 8.0, 64);
    let deferred_retry = source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 24.0, 64);
    let first_new = source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 40.0, 64);
    let deferred_new = source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 56.0, 64);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
        [
            queued_glyph(8, first_retry, 261),
            queued_glyph(9, deferred_retry, 261),
        ],
        [first_new, deferred_new],
        UVec2::new(32, 32),
        261,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_source_bytes_per_frame: Some(96),
            max_new_source_bytes_per_frame: Some(96),
            defer_excess_by_frames: 2,
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.retry_submission_report();

    assert_eq!(report.retried_source_byte_count, 64);
    assert_eq!(report.new_source_byte_count, 64);
    assert_eq!(report.deferred_retry_source_byte_count, 64);
    assert_eq!(report.backpressured_retry_source_byte_count, 64);
    assert_eq!(report.deferred_new_source_byte_count, 64);
    assert_eq!(report.backpressured_new_source_byte_count, 64);
    assert_eq!(report.next_retry_frame_index, Some(263));
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_report_exposes_terminal_byte_rejections() {
    let rejected_retry = source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 8.0, 97);
    let rejected_new = source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 24.0, 97);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
        [queued_glyph(4, rejected_retry, 271)],
        [rejected_new],
        UVec2::new(32, 32),
        271,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_source_bytes_per_frame: Some(96),
            max_new_source_bytes_per_frame: Some(96),
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = plan.retry_submission_report();

    assert_eq!(report.rejected_retry_source_count, 1);
    assert_eq!(report.rejected_retry_source_byte_count, 97);
    assert_eq!(report.rejected_new_source_count, 1);
    assert_eq!(report.rejected_new_source_byte_count, 97);
    assert!(report.has_byte_budget_rejections());
    assert!(plan.frame_outcome.next_blocked_glyphs.is_empty());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_keeps_resident_slots_out_of_new_work_budget() {
    let source = keyed_source(77, 8.0);
    let first = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [source],
        UVec2::new(32, 32),
        301,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let stable = glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding(
        first.run.atlas,
        [],
        [source],
        UVec2::new(32, 32),
        302,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_new_sources_per_frame: Some(0),
            max_new_source_bytes_per_frame: Some(0),
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(stable.frame_input.new_source_count, 1);
    assert_eq!(stable.frame_input.budgeted_new_source_count, 0);
    assert_eq!(stable.frame_input.budgeted_new_source_byte_count, 0);
    assert_eq!(stable.submission.run.glyphs.len(), 1);
    assert!(stable.submission.run.upload_copies.is_empty());
    assert!(stable.frame_outcome.next_blocked_glyphs.is_empty());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_budgets_duplicate_slot_misses_once() {
    let first = keyed_source(79, 8.0);
    let mut duplicate = first;
    duplicate.screen_rect.x = 24.0;

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding(
        GlyphAtlasSet::default(),
        [],
        [first, duplicate],
        UVec2::new(32, 32),
        311,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_new_sources_per_frame: Some(1),
            max_new_source_bytes_per_frame: Some(64),
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.frame_input.new_source_count, 2);
    assert_eq!(plan.frame_input.budgeted_new_source_count, 1);
    assert_eq!(plan.frame_input.budgeted_new_source_byte_count, 64);
    assert_eq!(plan.submission.run.glyphs.len(), 2);
    assert_eq!(plan.submission.run.upload_copies.len(), 1);
    assert!(plan.frame_outcome.next_blocked_glyphs.is_empty());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_does_not_bypass_a_rejected_duplicate_miss() {
    let first = keyed_source(81, 8.0);
    let mut duplicate = first;
    duplicate.screen_rect.x = 24.0;

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding(
        GlyphAtlasSet::default(),
        [],
        [first, duplicate],
        UVec2::new(32, 32),
        321,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_new_sources_per_frame: Some(0),
            max_new_source_bytes_per_frame: Some(0),
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert!(plan.frame_input.sources.is_empty());
    assert_eq!(plan.frame_input.rejected_new_source_count, 2);
    assert_eq!(plan.frame_input.rejected_new_source_byte_count, 128);
    assert!(plan.frame_outcome.next_blocked_glyphs.is_empty());
}

#[test]
fn render_text_atlas_bitmap_retry_frame_submission_deduplicates_retry_and_new_work_by_raster_key() {
    let source = keyed_source(83, 8.0);

    let plan = glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding(
        GlyphAtlasSet::default(),
        [queued_glyph(4, source, 331)],
        [source],
        UVec2::new(32, 32),
        331,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_sources_per_frame: Some(1),
            max_due_retry_source_bytes_per_frame: Some(64),
            max_new_sources_per_frame: Some(0),
            max_new_source_bytes_per_frame: Some(0),
            ..GlyphAtlasBitmapRetryBackpressurePolicy::unlimited()
        },
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.frame_input.retried_source_count, 1);
    assert_eq!(plan.frame_input.new_source_count, 1);
    assert_eq!(plan.frame_input.budgeted_new_source_count, 0);
    assert_eq!(plan.submission.run.glyphs.len(), 2);
    assert_eq!(plan.submission.run.upload_copies.len(), 1);
    assert!(plan.frame_outcome.next_blocked_glyphs.is_empty());
}

mod state;
