use super::*;
use crate::text::InstancedFaceId;
use crate::text::atlas::{
    GlyphAtlasRect, GlyphHintingMode, GlyphRasterKey, GlyphSmoothingMode, SyntheticGlyphStyle,
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
};

#[test]
fn render_perf_text_atlas_reuses_persistent_slot_without_upload() {
    let raster_key = raster_key(7);
    let first = glyph_atlas_bitmap_run_plan_with_padding(
        [keyed_source(
            raster_key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            4.0,
            64,
        )],
        UVec2::new(32, 32),
        20,
        1,
        2,
    );
    let first_rect = first.glyphs[0].atlas_rect;

    assert_eq!(first.slot_cache_hit_count, 0);
    assert_eq!(first.slot_cache_miss_count, 1);
    assert_eq!(first.slot_cache_insert_count, 1);
    assert_eq!(first.upload_copies.len(), 1);

    let mut moved = keyed_source(
        raster_key,
        GlyphAtlasFormat::AlphaMask,
        UVec2::new(8, 8),
        18.0,
        64,
    );
    moved.screen_rect.y = 14.0;
    let second = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        first.atlas,
        [moved],
        UVec2::new(32, 32),
        21,
        1,
        2,
    );

    assert_eq!(second.glyphs[0].atlas_rect, first_rect);
    assert_eq!(second.draw_glyphs[0].screen_rect, moved.screen_rect);
    assert_eq!(second.slot_cache_hit_count, 1);
    assert_eq!(second.slot_cache_miss_count, 0);
    assert_eq!(second.slot_cache_insert_count, 0);
    assert!(second.dirty_pages.is_empty());
    assert!(second.upload_copies.is_empty());
    assert!(second.upload_commands.is_empty());
}

#[test]
fn render_perf_text_atlas_persistent_slot_allocates_only_new_glyph() {
    let first_key = raster_key(11);
    let second_key = raster_key(12);
    let first = glyph_atlas_bitmap_run_plan_with_padding(
        [keyed_source(
            first_key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            4.0,
            64,
        )],
        UVec2::new(32, 32),
        30,
        1,
        2,
    );
    let first_rect = first.glyphs[0].atlas_rect;

    let second = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        first.atlas,
        [
            keyed_source(
                first_key,
                GlyphAtlasFormat::AlphaMask,
                UVec2::new(8, 8),
                4.0,
                64,
            ),
            keyed_source(
                second_key,
                GlyphAtlasFormat::AlphaMask,
                UVec2::new(8, 8),
                18.0,
                64,
            ),
        ],
        UVec2::new(32, 32),
        31,
        1,
        2,
    );

    assert_eq!(second.slot_cache_hit_count, 1);
    assert_eq!(second.slot_cache_miss_count, 1);
    assert_eq!(second.slot_cache_insert_count, 1);
    assert_eq!(second.glyphs[0].atlas_rect, first_rect);
    assert_ne!(second.glyphs[1].atlas_rect, first_rect);
    assert_eq!(second.upload_copies.len(), 1);
    assert_eq!(second.upload_copies[0].source_index, 1);
    assert_eq!(second.upload_commands.len(), 1);
}

#[test]
fn render_text_atlas_persistent_shadow_caps_new_upload_writes_without_erasing_old_glyphs() {
    let first_key = raster_key(13);
    let first_source = keyed_source(
        first_key,
        GlyphAtlasFormat::AlphaMask,
        UVec2::new(4, 4),
        4.0,
        16,
    );
    let first =
        glyph_atlas_bitmap_run_plan_with_padding([first_source], UVec2::new(64, 64), 32, 1, 2);
    let old_slot = first.glyphs[0].atlas_rect;
    let first_prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &first,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &[0xA5; 16])],
    );
    let first_shadow_commit = glyph_atlas_bitmap_page_shadow_commit(&first, first_prepared, true);
    let mut atlas = first.atlas;
    atlas.commit_bitmap_page_shadow(first_shadow_commit);

    let sources = std::iter::once(keyed_source(
        first_key,
        GlyphAtlasFormat::AlphaMask,
        UVec2::new(4, 4),
        4.0,
        16,
    ))
    .chain((0..9).map(|offset| {
        keyed_source(
            raster_key(14 + offset),
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(4, 4),
            12.0 + offset as f32 * 8.0,
            16,
        )
    }))
    .collect::<Vec<_>>();
    let next = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        atlas,
        sources,
        UVec2::new(64, 64),
        33,
        1,
        2,
    );

    assert_eq!(next.slot_cache_hit_count, 1);
    assert_eq!(next.slot_cache_miss_count, 9);
    assert_eq!(next.upload_copies.len(), 9);
    assert_eq!(next.upload_commands.len(), 1);
    assert!(next.upload_commands.len() <= 8);
    assert_eq!(
        next.upload_commands[0].rect,
        GlyphAtlasRect {
            x: 0,
            y: 0,
            width: 58,
            height: 4,
        }
    );
    let next_prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &next,
        (0..10)
            .map(|source_index| GlyphAtlasBitmapUploadSourceBytes::new(source_index, &[0x5A; 16])),
    );
    let staging = &next_prepared.staging.pages[0];
    assert_eq!(staging.target_rect, next.upload_commands[0].rect);
    assert_eq!(staging.bytes[old_slot.x as usize], 0xA5);
}

#[test]
fn render_text_atlas_full_page_replay_preserves_existing_persistent_slot() {
    let first_key = raster_key(23);
    let first_source = keyed_source(
        first_key,
        GlyphAtlasFormat::AlphaMask,
        UVec2::new(8, 8),
        4.0,
        64,
    );
    let first =
        glyph_atlas_bitmap_run_plan_with_padding([first_source], UVec2::new(64, 64), 43, 1, 0);
    let old_slot = first.glyphs[0].atlas_rect;
    let first_prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &first,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &[0xA5; 64])],
    );
    let first_shadow_commit = glyph_atlas_bitmap_page_shadow_commit(&first, first_prepared, true);
    let mut atlas = first.atlas;
    atlas.commit_bitmap_page_shadow(first_shadow_commit);

    let replacement = keyed_source(
        raster_key(24),
        GlyphAtlasFormat::AlphaMask,
        UVec2::new(56, 64),
        18.0,
        56 * 64,
    );
    let next = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        atlas,
        [first_source, replacement],
        UVec2::new(64, 64),
        44,
        1,
        0,
    );

    assert_eq!(next.slot_cache_hit_count, 1);
    assert_eq!(next.slot_cache_miss_count, 1);
    assert_eq!(next.upload_copies.len(), 1);
    assert_eq!(next.upload_commands.len(), 1);
    assert_eq!(
        next.upload_commands[0].rect,
        GlyphAtlasRect {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }
    );

    let next_prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &next,
        [GlyphAtlasBitmapUploadSourceBytes::new(1, &[0x5A; 56 * 64])],
    );
    let staging = &next_prepared.staging.pages[0];
    assert_eq!(staging.target_rect, next.upload_commands[0].rect);
    assert_eq!(staging.bytes[old_slot.x as usize], 0xA5);
    assert_eq!(
        staging.bytes[(old_slot.y as usize + old_slot.height as usize - 1) * 64
            + old_slot.x as usize
            + old_slot.width as usize
            - 1],
        0xA5,
    );
}

#[test]
fn physical_texture_replacement_replays_stable_committed_page_shadow() {
    let key = raster_key(25);
    let source = keyed_source(key, GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 4.0, 64);
    let first = glyph_atlas_bitmap_run_plan_with_padding([source], UVec2::new(32, 32), 45, 1, 0);
    let first_prepared = glyph_atlas_bitmap_prepared_upload_plan(
        &first,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &[0xA5; 64])],
    );
    let first_shadow_commit = glyph_atlas_bitmap_page_shadow_commit(&first, first_prepared, true);
    let mut atlas = first.atlas;
    atlas.commit_bitmap_page_shadow(first_shadow_commit);

    let stable = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        atlas,
        [source],
        UVec2::new(32, 32),
        46,
        1,
        0,
    );
    assert_eq!(stable.slot_cache_hit_count, 1);
    assert!(stable.upload_commands.is_empty());

    let replay = glyph_atlas_bitmap_prepared_upload_plan_with_full_shadow_replay(
        &stable,
        std::iter::empty::<GlyphAtlasBitmapUploadSourceBytes<'_>>(),
    );

    assert!(!replay.has_failures());
    assert_eq!(replay.staging.pages.len(), 1);
    assert_eq!(replay.staged_uploads.uploads.len(), 1);
    assert_eq!(
        replay.staged_uploads.uploads[0].command.mode,
        GlyphAtlasUploadMode::FullPage
    );
    assert_eq!(replay.staging.pages[0].bytes[0], 0xA5);
}

#[test]
fn render_perf_text_scroll_list_reuses_raster_slots_and_uploads_only_entering_rows() {
    const VISIBLE_ROW_COUNT: u32 = 5;
    const SCROLL_DELTA_ROWS: u32 = 3;
    const RASTER_BYTE_LEN: usize = 64;
    let page_size = UVec2::new(64, 64);
    let viewport_size = UVec2::new(128, 80);
    let clip_rect = GlyphAtlasScreenRect::new(0.0, 0.0, 128.0, 80.0);

    let first = glyph_atlas_bitmap_render_submission_plan(
        scroll_list_sources(0, VISIBLE_ROW_COUNT, RASTER_BYTE_LEN),
        page_size,
        32,
        1,
        viewport_size,
        clip_rect,
    );
    let first_report = first.submission_report();

    assert_eq!(first_report.source_count, VISIBLE_ROW_COUNT as usize);
    assert_eq!(first_report.slot_cache_hit_count, 0);
    assert_eq!(
        first_report.slot_cache_miss_count,
        VISIBLE_ROW_COUNT as usize
    );
    assert_eq!(
        first_report.upload_copy_byte_len,
        VISIBLE_ROW_COUNT as usize * RASTER_BYTE_LEN
    );

    let scrolled = glyph_atlas_bitmap_render_submission_plan_with_atlas(
        first.run.atlas,
        scroll_list_sources(SCROLL_DELTA_ROWS, VISIBLE_ROW_COUNT, RASTER_BYTE_LEN),
        page_size,
        33,
        1,
        viewport_size,
        clip_rect,
    );
    let scrolled_report = scrolled.submission_report();

    assert_ne!(
        first.run.draw_glyphs[SCROLL_DELTA_ROWS as usize]
            .screen_rect
            .y,
        scrolled.run.draw_glyphs[0].screen_rect.y,
        "overlapping rows must be allowed to move on screen without losing their raster slot"
    );
    assert_eq!(scrolled_report.source_count, VISIBLE_ROW_COUNT as usize);
    assert_eq!(
        scrolled_report.slot_cache_hit_count,
        (VISIBLE_ROW_COUNT - SCROLL_DELTA_ROWS) as usize
    );
    assert_eq!(
        scrolled_report.slot_cache_miss_count,
        SCROLL_DELTA_ROWS as usize
    );
    assert_eq!(
        scrolled_report.slot_cache_insert_count,
        SCROLL_DELTA_ROWS as usize
    );
    assert_eq!(
        scrolled_report.upload_copy_count,
        SCROLL_DELTA_ROWS as usize
    );
    assert_eq!(
        scrolled_report.upload_copy_byte_len,
        SCROLL_DELTA_ROWS as usize * RASTER_BYTE_LEN
    );
    assert!(scrolled_report.has_upload_copy_work());
    assert_eq!(
        scrolled
            .run
            .upload_copies
            .iter()
            .map(|copy| copy.source_index)
            .collect::<Vec<_>>(),
        vec![2, 3, 4],
        "only rows newly entering the viewport may contribute upload work"
    );
}

#[test]
fn render_text_atlas_same_frame_slot_projection_preserves_upload() {
    let key = raster_key(16);
    let source = keyed_source(key, GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 4.0, 64);
    let first = glyph_atlas_bitmap_run_plan_with_padding([source], UVec2::new(32, 32), 35, 1, 2);

    let projected = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        first.atlas,
        [source],
        UVec2::new(32, 32),
        35,
        1,
        2,
    );

    assert_eq!(projected.slot_cache_hit_count, 1);
    assert_eq!(projected.upload_copies.len(), 1);
    assert_eq!(projected.upload_commands.len(), 1);

    let stable = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        projected.atlas,
        [source],
        UVec2::new(32, 32),
        36,
        1,
        2,
    );

    assert_eq!(stable.slot_cache_hit_count, 1);
    assert!(stable.upload_copies.is_empty());
    assert!(stable.upload_commands.is_empty());
}

#[test]
fn render_text_atlas_duplicate_key_in_one_run_draws_twice_and_uploads_once() {
    let key = raster_key(17);
    let first = keyed_source(key, GlyphAtlasFormat::AlphaMask, UVec2::new(8, 8), 4.0, 64);
    let mut second = first;
    second.screen_rect.x = 18.0;

    let plan =
        glyph_atlas_bitmap_run_plan_with_padding([first, second], UVec2::new(8, 8), 36, 1, 0);

    assert_eq!(plan.glyphs.len(), 2);
    assert_eq!(plan.draw_glyphs.len(), 2);
    assert_eq!(plan.upload_copies.len(), 1);
    assert_eq!(plan.upload_commands.len(), 1);
    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.glyphs[0].atlas_rect, plan.glyphs[1].atlas_rect);
    assert_eq!(plan.glyphs[0].source_index, 0);
    assert_eq!(plan.glyphs[1].source_index, 1);
}

#[test]
fn render_perf_text_atlas_submission_reports_persistent_slot_counters() {
    let source = keyed_source(
        raster_key(18),
        GlyphAtlasFormat::AlphaMask,
        UVec2::new(8, 8),
        4.0,
        64,
    );
    let first = glyph_atlas_bitmap_render_submission_plan(
        [source],
        UVec2::new(32, 32),
        37,
        1,
        UVec2::new(64, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 32.0),
    );
    let first_report = first.submission_report();

    assert_eq!(first_report.slot_cache_hit_count, 0);
    assert_eq!(first_report.slot_cache_miss_count, 1);
    assert_eq!(first_report.slot_cache_insert_count, 1);

    let stable = glyph_atlas_bitmap_render_submission_plan_with_atlas(
        first.run.atlas,
        [source],
        UVec2::new(32, 32),
        38,
        1,
        UVec2::new(64, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 32.0),
    );
    let stable_report = stable.submission_report();

    assert_eq!(stable_report.slot_cache_hit_count, 1);
    assert_eq!(stable_report.slot_cache_miss_count, 0);
    assert_eq!(stable_report.slot_cache_insert_count, 0);
    assert_eq!(stable_report.upload_command_count, 0);
}

#[test]
fn render_text_atlas_persistent_slot_eviction_invalidates_page_identity() {
    let first_key = raster_key(21);
    let replacement_key = raster_key(22);
    let first = glyph_atlas_bitmap_run_plan_with_padding(
        [keyed_source(
            first_key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            4.0,
            64,
        )],
        UVec2::new(8, 8),
        40,
        1,
        0,
    );

    let replacement = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        first.atlas,
        [keyed_source(
            replacement_key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            14.0,
            64,
        )],
        UVec2::new(8, 8),
        41,
        1,
        0,
    );

    assert_eq!(replacement.slot_cache_hit_count, 0);
    assert_eq!(replacement.slot_cache_miss_count, 1);
    assert_eq!(replacement.rebuilt_pages.len(), 1);
    assert_eq!(replacement.slot_invalidations.len(), 1);
    assert_eq!(replacement.slot_invalidations[0].page_generation, 1);
    assert_eq!(replacement.upload_commands.len(), 1);
    assert_eq!(replacement.upload_commands[0].rect.width, 8);
    assert_eq!(replacement.upload_commands[0].rect.height, 8);
    assert_eq!(replacement.invalidated_raster_keys, vec![first_key]);

    let first_again = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        replacement.atlas,
        [keyed_source(
            first_key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            24.0,
            64,
        )],
        UVec2::new(8, 8),
        42,
        1,
        0,
    );

    assert_eq!(first_again.slot_cache_hit_count, 0);
    assert_eq!(first_again.slot_cache_miss_count, 1);
    assert_eq!(first_again.slot_cache_insert_count, 1);
    assert_eq!(first_again.slot_invalidations[0].page_generation, 2);
}

#[test]
fn render_text_atlas_upload_failure_deduplicates_page_generation_invalidation() {
    let first_key = raster_key(27);
    let second_key = raster_key(28);
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            keyed_source(
                first_key,
                GlyphAtlasFormat::AlphaMask,
                UVec2::new(8, 8),
                4.0,
                64,
            ),
            keyed_source(
                second_key,
                GlyphAtlasFormat::AlphaMask,
                UVec2::new(8, 8),
                14.0,
                64,
            ),
        ],
        UVec2::new(32, 32),
        45,
        1,
        2,
    );
    let page_key = plan.glyphs[0].page_key;
    let initial_generation = plan
        .atlas
        .page(page_key.format, page_key.page_index)
        .expect("persistent page")
        .generation;
    let mut atlas = plan.atlas;

    let mut invalidated = atlas.invalidate_bitmap_page_upload_state([page_key, page_key, page_key]);
    invalidated.sort_by_key(|key| key.glyph_id);

    assert_eq!(invalidated, vec![first_key, second_key]);
    assert_eq!(
        atlas
            .page(page_key.format, page_key.page_index)
            .expect("invalidated page remains reusable")
            .generation,
        initial_generation + 1
    );
}

#[test]
fn render_text_atlas_persistent_slot_rebuilds_when_page_size_changes() {
    let key = raster_key(31);
    let first = glyph_atlas_bitmap_run_plan_with_padding(
        [keyed_source(
            key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            4.0,
            64,
        )],
        UVec2::new(16, 16),
        50,
        1,
        0,
    );

    let resized = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        first.atlas,
        [keyed_source(
            key,
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 8),
            14.0,
            64,
        )],
        UVec2::new(32, 32),
        51,
        1,
        0,
    );

    assert_eq!(resized.slot_cache_hit_count, 0);
    assert_eq!(resized.slot_cache_miss_count, 1);
    assert_eq!(resized.slot_cache_insert_count, 1);
    assert_eq!(resized.rebuilt_pages.len(), 1);
    assert_eq!(resized.upload_copies.len(), 1);
    assert_eq!(resized.draw_glyphs[0].atlas_size, UVec2::new(32, 32));
}

fn keyed_source(
    raster_key: GlyphRasterKey,
    format: GlyphAtlasFormat,
    content_size: UVec2,
    x: f32,
    source_byte_len: usize,
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        raster_key: Some(raster_key),
        ..source(format, content_size, x, source_byte_len)
    }
}

fn scroll_list_sources(
    first_row: u32,
    visible_row_count: u32,
    source_byte_len: usize,
) -> Vec<GlyphAtlasBitmapSource> {
    (0..visible_row_count)
        .map(|viewport_row| {
            let mut source = keyed_source(
                raster_key(first_row + viewport_row + 100),
                GlyphAtlasFormat::AlphaMask,
                UVec2::new(8, 8),
                8.0,
                source_byte_len,
            );
            source.screen_rect.y = 4.0 + viewport_row as f32 * 12.0;
            source
        })
        .collect()
}

fn raster_key(glyph_id: u32) -> GlyphRasterKey {
    GlyphRasterKey {
        face: InstancedFaceId(41),
        glyph_id,
        px_size_bucket: 16,
        subpixel_bin: 0,
        vertical_subpixel_bin: 0,
        format: GlyphAtlasFormat::AlphaMask,
        hinting: GlyphHintingMode::Full,
        smoothing: GlyphSmoothingMode::Grayscale,
        synthetic: SyntheticGlyphStyle::default(),
    }
}
