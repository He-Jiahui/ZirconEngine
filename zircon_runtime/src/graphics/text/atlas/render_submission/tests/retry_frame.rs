use super::*;

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
            defer_excess_by_frames: 3,
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
fn render_text_atlas_bitmap_retry_frame_state_starts_empty_and_drains_successful_retries() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let mut state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([queued_glyph(21, retry_source, 151)]);

    assert_eq!(state.queued_blocked_glyph_count(), 1);
    assert_eq!(state.next_retry_frame_index(), Some(151));
    assert!(state.report().has_queued_retry_work());

    let plan = state.submission_plan_with_padding(
        [],
        UVec2::new(32, 32),
        151,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.frame_input.retried_source_count, 1);
    assert_eq!(plan.frame_outcome.completed_retried_source_count, 1);
    assert_eq!(plan.frame_outcome.next_blocked_glyphs.len(), 0);

    let report = state.apply_submission_plan(&plan);

    assert_eq!(report.queued_blocked_glyph_count, 0);
    assert_eq!(report.next_retry_frame_index, None);
    assert!(!report.has_queued_retry_work());
    assert_eq!(state.queued_blocked_glyphs(), &[]);
}

#[test]
fn render_text_atlas_bitmap_retry_frame_state_persists_deferred_and_backpressured_retries() {
    let first_retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let backpressured_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);
    let deferred_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 40.0, 64);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 56.0, 96);
    let mut state = GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([
        queued_glyph(31, first_retry_source, 171),
        queued_glyph(32, backpressured_source, 171),
        queued_glyph(33, deferred_source, 173),
    ]);

    let plan = state.submission_plan_with_backpressure_and_padding(
        [new_source],
        UVec2::new(32, 32),
        171,
        1,
        2,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_sources_per_frame: Some(1),
            defer_excess_by_frames: 5,
        },
        UVec2::new(96, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 96.0, 32.0),
    );

    assert_eq!(
        plan.frame_input.sources,
        vec![first_retry_source, new_source]
    );
    assert_eq!(plan.frame_input.retried_source_count, 1);
    assert_eq!(plan.frame_input.new_source_count, 1);
    assert_eq!(plan.frame_input.deferred_retry_count, 2);
    assert_eq!(plan.frame_input.backpressured_retry_count, 1);

    let report = state.apply_submission_plan(&plan);

    assert_eq!(report.queued_blocked_glyph_count, 2);
    assert_eq!(report.next_retry_frame_index, Some(173));
    assert_eq!(
        state.queued_blocked_glyphs(),
        &[
            queued_glyph(32, backpressured_source, 176),
            queued_glyph(33, deferred_source, 173),
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_retry_frame_state_commits_new_blocked_sources() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let new_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);
    let mut state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([queued_glyph(41, retry_source, 191)]);

    let plan = state.submission_plan_with_padding(
        [new_source],
        UVec2::new(32, 32),
        191,
        0,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = state.apply_submission_plan(&plan);

    assert_eq!(report.queued_blocked_glyph_count, 2);
    assert_eq!(report.next_retry_frame_index, Some(192));
    assert_eq!(
        state.queued_blocked_glyphs(),
        &[
            queued_glyph(41, retry_source, 192),
            queued_glyph(0, new_source, 192),
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_retry_frame_driver_commits_successful_frame_state() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96);
    let mut state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([queued_glyph(51, retry_source, 211)]);

    let output = glyph_atlas_bitmap_retry_frame_driver_submit_with_config(
        &mut state,
        [new_source],
        211,
        frame_driver_config(UVec2::new(32, 32), 1, 2),
    );
    let retry_report = output.retry_submission_report();

    assert_eq!(
        output.submission_plan.frame_input.sources.as_slice(),
        &[retry_source, new_source]
    );
    assert_eq!(retry_report.retried_source_count, 1);
    assert_eq!(retry_report.new_source_count, 1);
    assert_eq!(retry_report.completed_retried_source_count, 1);
    assert_eq!(retry_report.completed_new_source_count, 1);
    assert_eq!(output.state_report.queued_blocked_glyph_count, 0);
    assert_eq!(output.state_report.next_retry_frame_index, None);
    assert_eq!(state.queued_blocked_glyphs(), &[]);
}

#[test]
fn render_text_atlas_bitmap_retry_frame_driver_applies_backpressure_and_commits_queue() {
    let first_retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let backpressured_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);
    let deferred_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 40.0, 64);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 56.0, 96);
    let mut state = GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([
        queued_glyph(61, first_retry_source, 221),
        queued_glyph(62, backpressured_source, 221),
        queued_glyph(63, deferred_source, 223),
    ]);

    let output = glyph_atlas_bitmap_retry_frame_driver_submit_with_config(
        &mut state,
        [new_source],
        221,
        GlyphAtlasBitmapRetryFrameDriverConfig {
            backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy {
                max_due_retry_sources_per_frame: Some(1),
                defer_excess_by_frames: 4,
            },
            ..frame_driver_config(UVec2::new(32, 32), 1, 2)
        },
    );
    let retry_report = output.retry_submission_report();

    assert_eq!(retry_report.retried_source_count, 1);
    assert_eq!(retry_report.new_source_count, 1);
    assert_eq!(retry_report.deferred_retry_count, 2);
    assert_eq!(retry_report.backpressured_retry_count, 1);
    assert_eq!(retry_report.completed_retried_source_count, 1);
    assert_eq!(retry_report.completed_new_source_count, 1);
    assert_eq!(output.state_report.queued_blocked_glyph_count, 2);
    assert_eq!(output.state_report.next_retry_frame_index, Some(223));
    assert_eq!(
        state.queued_blocked_glyphs(),
        &[
            queued_glyph(62, backpressured_source, 225),
            queued_glyph(63, deferred_source, 223),
        ]
    );
}
