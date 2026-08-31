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
fn sdf_atlas_run_keys_share_font_family_and_language_identity() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    let mut text = text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0));
    text.font = Some("res://fonts/default.font.toml".to_string());
    text.font_family = Some("Studio Mono".to_string());
    text.language = Some("zh_hans".to_string());

    atlas.prepare(&[text]);

    let first = &atlas.plan().slots[0].key;
    let second = &atlas.plan().slots[1].key;
    assert!(std::sync::Arc::ptr_eq(
        first.font.as_ref().expect("first font identity"),
        second.font.as_ref().expect("second font identity")
    ));
    assert!(std::sync::Arc::ptr_eq(
        first.font_family.as_ref().expect("first family identity"),
        second.font_family.as_ref().expect("second family identity")
    ));
    assert!(std::sync::Arc::ptr_eq(
        first.language.as_ref().expect("first language identity"),
        second.language.as_ref().expect("second language identity")
    ));
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
fn sdf_atlas_owner_reuses_compiled_plan_storage_for_identical_text_inputs() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    let text = text_batch("Stable", UiFrame::new(0.0, 0.0, 72.0, 12.0));

    atlas.prepare(std::slice::from_ref(&text));
    let slot_storage = atlas.plan().slots.as_ptr();
    let run_storage = atlas.plan().runs[0].glyph_slot_indices.as_ptr();
    atlas.prepare(std::slice::from_ref(&text));

    assert_eq!(atlas.plan().slots.as_ptr(), slot_storage);
    assert_eq!(
        atlas.plan().runs[0].glyph_slot_indices.as_ptr(),
        run_storage
    );
    assert_eq!(atlas.cache_report().added_slot_count, 0);
    assert_eq!(atlas.cache_report().relocated_slot_count, 0);
    assert!(atlas.cache_report().dirty_pages.is_empty());

    let changed = text_batch("Changed", UiFrame::new(0.0, 0.0, 84.0, 12.0));
    atlas.prepare(&[changed]);
    assert!(atlas.plan().slots.iter().any(|slot| slot.key.glyph == 'C'));
}

#[test]
fn sdf_atlas_owner_rebuilds_for_text_owned_glyph_identity() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    let first = artifact_text_batch(0xfb01, UiTextWritingMode::HorizontalTb);

    atlas.prepare(std::slice::from_ref(&first));
    let first_generation = atlas.generation;
    assert_eq!(atlas.plan().runs[0].glyph_slot_indices.len(), 1);
    assert_eq!(atlas.plan().slots[0].key.glyph_id, Some(0xfb01));

    let replacement = artifact_text_batch(0xfb02, UiTextWritingMode::HorizontalTb);
    atlas.prepare(std::slice::from_ref(&replacement));

    assert_eq!(atlas.generation, first_generation + 1);
    let slot_index = atlas.plan().runs[0].glyph_slot_indices[0].expect("artifact atlas slot");
    assert_eq!(atlas.plan().slots[slot_index].key.glyph_id, Some(0xfb02));
}

#[test]
fn sdf_atlas_owner_rebuilds_for_refreshed_text_owned_glyph_line() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    let (original, refreshed) = republished_artifact_text_batch();

    atlas.prepare(std::slice::from_ref(&original));
    let first_generation = atlas.generation;
    atlas.prepare(std::slice::from_ref(&refreshed));

    assert_eq!(atlas.generation, first_generation + 1);
    let slot_index = atlas.plan().runs[0].glyph_slot_indices[0].expect("refreshed atlas slot");
    assert_eq!(atlas.plan().slots[slot_index].key.glyph_id, Some(0xfb02));
}

#[test]
fn sdf_atlas_owner_rebuilds_for_text_owned_writing_mode_change() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    let horizontal = artifact_text_batch(0xfb01, UiTextWritingMode::HorizontalTb);

    atlas.prepare(std::slice::from_ref(&horizontal));
    let first_generation = atlas.generation;
    let mut vertical = horizontal.clone();
    vertical.writing_mode = UiTextWritingMode::VerticalRl;

    atlas.prepare(std::slice::from_ref(&vertical));

    assert_eq!(atlas.generation, first_generation + 1);
    assert_eq!(atlas.plan().runs[0].glyph_slot_indices.len(), 1);
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
