use super::*;

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
            dirty_rect: Some(sdf_rect(0, 0, 128, 64)),
            dirty_pages: dirty_pages(&[sdf_rect(0, 0, 128, 64)]),
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
            dirty_rect: Some(sdf_rect(128, 0, 64, 64)),
            dirty_pages: dirty_pages(&[sdf_rect(128, 0, 64, 64)]),
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
            dirty_rect: Some(sdf_rect(128, 0, 64, 64)),
            dirty_pages: dirty_pages(&[sdf_rect(128, 0, 64, 64)]),
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
            dirty_rect: None,
            dirty_pages: Vec::new(),
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
            dirty_rect: None,
            dirty_pages: Vec::new(),
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
            dirty_rect: None,
            dirty_pages: Vec::new(),
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
            dirty_rect: Some(sdf_rect(0, 0, 512, 512)),
            dirty_pages: dirty_pages_for_indices(&[0, 1, 2, 3], sdf_rect(0, 0, 512, 512)),
        }
    );
}
