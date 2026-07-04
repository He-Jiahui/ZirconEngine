use super::*;

#[test]
fn render_text_atlas_bitmap_retry_plan_splits_due_and_deferred_glyphs() {
    let due_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let later_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 24.0, 64);
    let retry_plan = glyph_atlas_bitmap_retry_plan(
        [
            queued_glyph(2, due_source, 31),
            queued_glyph(5, later_source, 33),
        ],
        31,
    );
    let retry_sources = retry_plan.retry_sources().collect::<Vec<_>>();

    assert_eq!(retry_plan.due_retry_count, 1);
    assert_eq!(retry_plan.deferred_retry_count, 1);
    assert_eq!(retry_plan.next_retry_frame_index, Some(33));
    assert_eq!(retry_plan.retry_glyphs[0].source_index, 2);
    assert_eq!(retry_plan.deferred_glyphs[0].source_index, 5);
    assert_eq!(retry_sources, vec![due_source]);
}

#[test]
fn render_text_atlas_bitmap_retry_plan_retries_equal_frame_index() {
    let source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 18.0, 96);
    let retry_plan = glyph_atlas_bitmap_retry_plan([queued_glyph(7, source, 41)], 41);

    assert_eq!(retry_plan.due_retry_count, 1);
    assert_eq!(retry_plan.deferred_retry_count, 0);
    assert_eq!(retry_plan.next_retry_frame_index, None);
    assert_eq!(retry_plan.retry_sources().collect::<Vec<_>>(), vec![source]);
}

#[test]
fn render_text_atlas_bitmap_retry_plan_backpressures_due_retries_over_budget() {
    let first_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let backpressured_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 24.0, 96);
    let later_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 40.0, 64);
    let retry_plan = glyph_atlas_bitmap_retry_plan_with_backpressure(
        [
            queued_glyph(2, first_source, 91),
            queued_glyph(5, backpressured_source, 90),
            queued_glyph(7, later_source, 95),
        ],
        91,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_sources_per_frame: Some(1),
            defer_excess_by_frames: 2,
        },
    );

    assert_eq!(retry_plan.due_retry_count, 1);
    assert_eq!(retry_plan.deferred_retry_count, 2);
    assert_eq!(retry_plan.backpressured_retry_count, 1);
    assert_eq!(retry_plan.next_retry_frame_index, Some(93));
    assert_eq!(
        retry_plan.retry_glyphs,
        vec![queued_glyph(2, first_source, 91)]
    );
    assert_eq!(
        retry_plan.deferred_glyphs,
        vec![
            queued_glyph(5, backpressured_source, 93),
            queued_glyph(7, later_source, 95),
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_retry_frame_input_merges_due_retries_before_new_sources() {
    let due_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let deferred_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 24.0, 64);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 40.0, 96);

    let frame_input = glyph_atlas_bitmap_retry_frame_input(
        [
            queued_glyph(9, due_source, 51),
            queued_glyph(11, deferred_source, 53),
        ],
        [new_source],
        51,
    );

    assert_eq!(frame_input.sources, vec![due_source, new_source]);
    assert_eq!(
        frame_input.source_origins,
        vec![
            GlyphAtlasBitmapRetrySourceOrigin::Retried {
                source_index: 9,
                retry_frame_index: 51,
            },
            GlyphAtlasBitmapRetrySourceOrigin::New { source_index: 0 },
        ]
    );
    assert_eq!(
        frame_input.deferred_glyphs,
        vec![queued_glyph(11, deferred_source, 53)]
    );
    assert_eq!(frame_input.retried_source_count, 1);
    assert_eq!(frame_input.new_source_count, 1);
    assert_eq!(frame_input.deferred_retry_count, 1);
    assert_eq!(frame_input.next_retry_frame_index, Some(53));
}

#[test]
fn render_text_atlas_bitmap_retry_frame_input_applies_backpressure_before_new_sources() {
    let due_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let backpressured_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 40.0, 96);

    let frame_input = glyph_atlas_bitmap_retry_frame_input_with_backpressure(
        [
            queued_glyph(9, due_source, 101),
            queued_glyph(11, backpressured_source, 101),
        ],
        [new_source],
        101,
        GlyphAtlasBitmapRetryBackpressurePolicy {
            max_due_retry_sources_per_frame: Some(1),
            defer_excess_by_frames: 1,
        },
    );

    assert_eq!(frame_input.sources, vec![due_source, new_source]);
    assert_eq!(frame_input.retried_source_count, 1);
    assert_eq!(frame_input.new_source_count, 1);
    assert_eq!(frame_input.deferred_retry_count, 1);
    assert_eq!(frame_input.backpressured_retry_count, 1);
    assert_eq!(frame_input.next_retry_frame_index, Some(102));
    assert_eq!(
        frame_input.deferred_glyphs,
        vec![queued_glyph(11, backpressured_source, 102)]
    );
    assert_eq!(
        frame_input.source_origins,
        vec![
            GlyphAtlasBitmapRetrySourceOrigin::Retried {
                source_index: 9,
                retry_frame_index: 101,
            },
            GlyphAtlasBitmapRetrySourceOrigin::New { source_index: 0 },
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_retry_frame_input_origin_map_tracks_run_local_indices() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let new_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 24.0, 36);
    let frame_input =
        glyph_atlas_bitmap_retry_frame_input([queued_glyph(4, retry_source, 61)], [new_source], 61);

    let run_plan = glyph_atlas_bitmap_run_plan_with_padding(
        frame_input.sources.clone(),
        UVec2::new(32, 32),
        61,
        1,
        2,
    );

    assert_eq!(run_plan.glyphs.len(), 2);
    assert_eq!(
        frame_input.source_origins[run_plan.glyphs[0].source_index],
        GlyphAtlasBitmapRetrySourceOrigin::Retried {
            source_index: 4,
            retry_frame_index: 61,
        }
    );
    assert_eq!(
        frame_input.source_origins[run_plan.glyphs[1].source_index],
        GlyphAtlasBitmapRetrySourceOrigin::New { source_index: 0 }
    );
}

#[test]
fn render_text_atlas_bitmap_retry_frame_outcome_preserves_deferred_queue_after_success() {
    let due_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let deferred_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 24.0, 64);
    let new_source = source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 40.0, 96);
    let deferred = queued_glyph(11, deferred_source, 73);
    let frame_input = glyph_atlas_bitmap_retry_frame_input(
        [queued_glyph(9, due_source, 71), deferred],
        [new_source],
        71,
    );
    let run_plan = glyph_atlas_bitmap_run_plan_with_padding(
        frame_input.sources.clone(),
        UVec2::new(32, 32),
        71,
        1,
        2,
    );

    let outcome = glyph_atlas_bitmap_retry_frame_outcome(&frame_input, &run_plan);

    assert_eq!(outcome.next_blocked_glyphs, vec![deferred]);
    assert_eq!(outcome.completed_retried_source_count, 1);
    assert_eq!(outcome.completed_new_source_count, 1);
    assert_eq!(outcome.blocked_retried_source_count, 0);
    assert_eq!(outcome.blocked_new_source_count, 0);
    assert_eq!(outcome.deferred_retry_count, 1);
    assert_eq!(outcome.unmapped_blocked_source_count, 0);
    assert_eq!(outcome.next_retry_frame_index, Some(73));
}

#[test]
fn render_text_atlas_bitmap_retry_frame_outcome_remaps_blocked_run_sources() {
    let retry_source = source(GlyphAtlasFormat::AlphaMask, UVec2::new(6, 6), 8.0, 36);
    let new_source = source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 24.0, 64);
    let frame_input = glyph_atlas_bitmap_retry_frame_input(
        [queued_glyph(14, retry_source, 81)],
        [new_source],
        81,
    );
    let run_plan = glyph_atlas_bitmap_run_plan_with_padding(
        frame_input.sources.clone(),
        UVec2::new(32, 32),
        81,
        0,
        2,
    );

    let outcome = glyph_atlas_bitmap_retry_frame_outcome(&frame_input, &run_plan);

    assert!(run_plan.glyphs.is_empty());
    assert_eq!(run_plan.blocked_glyphs.len(), 2);
    assert_eq!(run_plan.blocked_glyphs[0].source_index, 0);
    assert_eq!(run_plan.blocked_glyphs[1].source_index, 1);
    assert_eq!(outcome.next_blocked_glyphs[0].source_index, 14);
    assert_eq!(outcome.next_blocked_glyphs[0].retry_frame_index, 82);
    assert_eq!(outcome.next_blocked_glyphs[1].source_index, 0);
    assert_eq!(outcome.next_blocked_glyphs[1].retry_frame_index, 82);
    assert_eq!(outcome.completed_retried_source_count, 0);
    assert_eq!(outcome.completed_new_source_count, 0);
    assert_eq!(outcome.blocked_retried_source_count, 1);
    assert_eq!(outcome.blocked_new_source_count, 1);
    assert_eq!(outcome.deferred_retry_count, 0);
    assert_eq!(outcome.unmapped_blocked_source_count, 0);
    assert_eq!(outcome.next_retry_frame_index, Some(82));
}
