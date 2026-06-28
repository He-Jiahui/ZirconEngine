use super::*;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWrap};

#[test]
fn sdf_atlas_plan_deduplicates_glyph_slots_across_batches() {
    let plan = plan_sdf_atlas(&[
        text_batch("ABBA", UiFrame::new(10.0, 20.0, 40.0, 12.0)),
        text_batch("CAB", UiFrame::new(10.0, 36.0, 30.0, 12.0)),
    ]);

    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(512));
    assert_eq!(plan.slots.len(), 3);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(plan.slots[0].rect.x, 0);
    assert_eq!(plan.slots[0].rect.y, 0);
    assert_eq!(plan.slots[1].key.glyph, 'B');
    assert_eq!(plan.slots[1].rect.x, 64);
    assert_eq!(plan.slots[1].rect.y, 0);
    assert_eq!(plan.slots[2].key.glyph, 'C');
    assert_eq!(plan.slots[2].rect.x, 128);
    assert_eq!(plan.slots[2].rect.y, 0);
    assert_eq!(plan.runs.len(), 2);
    assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0, 1, 1, 0]));
    assert_eq!(plan.runs[1].glyph_slot_indices, glyph_slots(&[2, 0, 1]));
}

#[test]
fn sdf_atlas_plan_keys_glyph_slots_by_font_identity_and_size() {
    let mut small_default = text_batch("A", UiFrame::new(0.0, 0.0, 12.0, 12.0));
    small_default.font_size = 12.0;
    let mut large_default = text_batch("A", UiFrame::new(0.0, 16.0, 24.0, 24.0));
    large_default.font_size = 24.0;
    let mut small_icon = text_batch("A", UiFrame::new(0.0, 48.0, 12.0, 12.0));
    small_icon.font = Some("res://fonts/icons.font.toml".to_string());
    small_icon.font_family = Some("Zircon Icons".to_string());

    let plan = plan_sdf_atlas(&[small_default, large_default, small_icon]);

    assert_eq!(plan.slots.len(), 3);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(
        plan.slots[0].key.font.as_deref(),
        Some("res://fonts/default.font.toml")
    );
    assert_eq!(
        plan.slots[0].key.font_family.as_deref(),
        Some("Zircon Sans")
    );
    assert_eq!(plan.slots[0].key.font_size_milli, 12_000);
    assert_eq!(plan.slots[1].key.glyph, 'A');
    assert_eq!(plan.slots[1].key.font_size_milli, 24_000);
    assert_eq!(plan.slots[2].key.glyph, 'A');
    assert_eq!(
        plan.slots[2].key.font.as_deref(),
        Some("res://fonts/icons.font.toml")
    );
    assert_eq!(
        plan.slots[2].key.font_family.as_deref(),
        Some("Zircon Icons")
    );
    assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0]));
    assert_eq!(plan.runs[1].glyph_slot_indices, glyph_slots(&[1]));
    assert_eq!(plan.runs[2].glyph_slot_indices, glyph_slots(&[2]));
}

#[test]
fn sdf_atlas_plan_preserves_whitespace_advances_without_slots() {
    let plan = plan_sdf_atlas(&[text_batch("A B", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);

    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(plan.slots[1].key.glyph, 'B');
    assert_eq!(
        plan.runs[0].glyph_slot_indices,
        vec![Some(0), None, Some(1)]
    );
}

#[test]
fn sdf_atlas_plan_assigns_slot_rects_by_key_not_batch_order() {
    let first = plan_sdf_atlas(&[text_batch("AB", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);
    let second = plan_sdf_atlas(&[text_batch("BA", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);

    assert_eq!(first.slots, second.slots);
}

#[test]
fn sdf_atlas_quality_controls_slot_size_and_min_grid() {
    let plan = plan_sdf_atlas_with_quality(
        &[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))],
        SdfAtlasQuality {
            slot_size_px: 32,
            min_grid_side: 2,
            max_cached_slot_count: 8,
        },
    );

    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(64));
    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].rect.width, 32);
    assert_eq!(plan.slots[0].rect.height, 32);
    assert_eq!(plan.slots[1].rect.x, 32);
    assert_eq!(plan.slots[1].rect.y, 0);
}

#[test]
fn sdf_atlas_owner_retains_inactive_slots_between_non_empty_frames() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();

    atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
    assert_eq!(atlas.slot_count(), 2);
    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: 0,
            current_slot_count: 2,
            retained_slot_count: 0,
            stable_slot_count: 0,
            relocated_slot_count: 0,
            added_slot_count: 2,
            evicted_slot_count: 0,
            atlas_resized: true,
        }
    );

    atlas.prepare(&[text_batch("C", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);
    assert_eq!(atlas.slot_count(), 3);
    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: 2,
            current_slot_count: 3,
            retained_slot_count: 2,
            stable_slot_count: 2,
            relocated_slot_count: 0,
            added_slot_count: 1,
            evicted_slot_count: 0,
            atlas_resized: false,
        }
    );
}

#[test]
fn sdf_atlas_owner_reports_retained_and_added_slots() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();

    atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
    atlas.prepare(&[text_batch("BC", UiFrame::new(0.0, 16.0, 24.0, 12.0))]);

    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: 2,
            current_slot_count: 3,
            retained_slot_count: 2,
            stable_slot_count: 2,
            relocated_slot_count: 0,
            added_slot_count: 1,
            evicted_slot_count: 0,
            atlas_resized: false,
        }
    );
}

#[test]
fn sdf_atlas_owner_reuses_retained_slot_without_readding_glyph() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();

    atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
    atlas.prepare(&[text_batch("C", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);
    atlas.prepare(&[text_batch("A", UiFrame::new(0.0, 32.0, 12.0, 12.0))]);

    assert_eq!(atlas.slot_count(), 3);
    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: 3,
            current_slot_count: 3,
            retained_slot_count: 3,
            stable_slot_count: 3,
            relocated_slot_count: 0,
            added_slot_count: 0,
            evicted_slot_count: 0,
            atlas_resized: false,
        }
    );
    assert_eq!(atlas.plan().runs[0].glyph_slot_indices, glyph_slots(&[0]));
}

#[test]
fn sdf_atlas_owner_clears_previous_plan_for_native_only_frames() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();

    atlas.prepare(&[text_batch("SDF", UiFrame::new(0.0, 0.0, 36.0, 12.0))]);
    assert_eq!(atlas.slot_count(), 3);
    assert_eq!(atlas.run_count(), 1);

    atlas.prepare(&[]);
    assert_eq!(atlas.slot_count(), 0);
    assert_eq!(atlas.run_count(), 0);
    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: 3,
            current_slot_count: 0,
            retained_slot_count: 0,
            stable_slot_count: 0,
            relocated_slot_count: 0,
            added_slot_count: 0,
            evicted_slot_count: 3,
            atlas_resized: true,
        }
    );
}

#[test]
fn sdf_atlas_owner_preserves_whitespace_runs_without_cache_slots() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();

    atlas.prepare(&[text_batch("A", UiFrame::new(0.0, 0.0, 12.0, 12.0))]);
    atlas.prepare(&[text_batch("  ", UiFrame::new(0.0, 16.0, 24.0, 12.0))]);

    assert_eq!(atlas.slot_count(), 0);
    assert_eq!(atlas.run_count(), 1);
    assert_eq!(atlas.plan().runs[0].glyph_slot_indices, vec![None, None]);
    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: 1,
            current_slot_count: 0,
            retained_slot_count: 0,
            stable_slot_count: 0,
            relocated_slot_count: 0,
            added_slot_count: 0,
            evicted_slot_count: 1,
            atlas_resized: true,
        }
    );
}

#[test]
fn sdf_atlas_owner_evicts_old_inactive_slots_when_cache_limit_is_exceeded() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    let full_cache = glyph_range_string(0x1000, SDF_ATLAS_MAX_CACHED_SLOT_COUNT);

    atlas.prepare(&[text_batch(
        &full_cache,
        UiFrame::new(0.0, 0.0, 4096.0, 12.0),
    )]);
    assert_eq!(atlas.slot_count(), SDF_ATLAS_MAX_CACHED_SLOT_COUNT);

    atlas.prepare(&[text_batch("\u{2200}", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);

    assert_eq!(atlas.slot_count(), SDF_ATLAS_MAX_CACHED_SLOT_COUNT);
    assert_eq!(
        atlas.cache_report(),
        SdfAtlasCacheReport {
            previous_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT,
            current_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT,
            retained_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT - 1,
            stable_slot_count: 0,
            relocated_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT - 1,
            added_slot_count: 1,
            evicted_slot_count: 1,
            atlas_resized: false,
        }
    );
}

#[test]
fn sdf_atlas_plan_grows_to_fit_more_than_default_grid() {
    let many_glyphs = (0..70)
        .map(|index| char::from_u32(0x1000 + index).unwrap())
        .collect::<String>();

    let plan = plan_sdf_atlas(&[text_batch(
        &many_glyphs,
        UiFrame::new(0.0, 0.0, 4096.0, 12.0),
    )]);

    assert_eq!(plan.slots.len(), 70);
    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(1024));
    assert_eq!(plan.slots[64].rect.x, 0);
    assert_eq!(plan.slots[64].rect.y, 256);
}

fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame,
        clip_frame: None,
        color: [1.0, 1.0, 1.0, 1.0],
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        font_size: 12.0,
        line_height: 14.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        wrap: UiTextWrap::None,
        style: Default::default(),
    }
}

fn glyph_slots(indices: &[usize]) -> Vec<Option<usize>> {
    indices.iter().copied().map(Some).collect()
}

fn glyph_range_string(start: u32, count: usize) -> String {
    (0..count)
        .map(|index| char::from_u32(start + index as u32).unwrap())
        .collect()
}
