use super::*;

#[test]
fn sdf_atlas_cache_report_tracks_dirty_rects_by_page_key() {
    let previous = synthetic_plan(vec![
        slot_on_page('A', 0, sdf_rect(0, 0, 64, 64)),
        slot_on_page('B', 1, sdf_rect(0, 0, 64, 64)),
    ]);
    let current = synthetic_plan(vec![
        slot_on_page('A', 0, sdf_rect(0, 0, 64, 64)),
        slot_on_page('B', 1, sdf_rect(64, 0, 64, 64)),
        slot_on_page('C', 0, sdf_rect(128, 0, 64, 64)),
    ]);

    let report = cache_report_for_plan_transition(&previous, &current);

    assert_eq!(report.retained_slot_count, 2);
    assert_eq!(report.stable_slot_count, 1);
    assert_eq!(report.relocated_slot_count, 1);
    assert_eq!(report.added_slot_count, 1);
    assert_eq!(report.dirty_rect, Some(sdf_rect(128, 0, 64, 64)));
    assert_eq!(
        report.dirty_pages,
        vec![
            dirty_page(0, sdf_rect(128, 0, 64, 64)),
            dirty_page(1, sdf_rect(64, 0, 64, 64)),
        ]
    );
}

#[test]
fn sdf_atlas_cache_report_marks_rebuilt_pages_full_dirty() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1);
    let previous = synthetic_plan(vec![slot_on_page('A', 1, sdf_rect(0, 0, 64, 64))]);
    let current = synthetic_plan_with_rebuilt_pages(
        vec![slot_on_page('A', 1, sdf_rect(0, 0, 64, 64))],
        vec![page_key],
    );

    let report = cache_report_for_plan_transition(&previous, &current);

    assert_eq!(report.retained_slot_count, 1);
    assert_eq!(report.stable_slot_count, 1);
    assert_eq!(report.relocated_slot_count, 0);
    assert_eq!(report.added_slot_count, 0);
    assert_eq!(report.evicted_slot_count, 0);
    assert_eq!(report.dirty_rect, None);
    assert_eq!(
        report.dirty_pages,
        vec![dirty_page(1, sdf_rect(0, 0, 256, 256))]
    );
}

#[test]
fn font_face_invalidation_rebuilds_stable_slots_as_dirty_pages() {
    let texts = [text_batch("AB", UiFrame::new(0.0, 0.0, 128.0, 32.0))];
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    atlas.prepare(&texts);
    atlas.prepare(&texts);
    assert_eq!(atlas.cache_report().stable_slot_count, 2);
    assert!(atlas.cache_report().dirty_pages.is_empty());

    atlas.invalidate_font_faces();
    atlas.prepare(&[]);
    atlas.mark_prepared_pages_uploaded();
    atlas.prepare(&texts);

    let report = atlas.cache_report();
    assert_eq!(report.previous_slot_count, 0);
    assert_eq!(report.current_slot_count, 2);
    assert_eq!(report.stable_slot_count, 0);
    assert_eq!(report.added_slot_count, 2);
    assert!(!report.dirty_pages.is_empty());
    assert!(report.dirty_pages.iter().all(|page| {
        page.dirty_rect.x == 0
            && page.dirty_rect.y == 0
            && page.dirty_rect.width == atlas.plan().atlas_size.x
            && page.dirty_rect.height == atlas.plan().atlas_size.y
    }));

    atlas.prepare(&texts);
    assert!(!atlas.cache_report().dirty_pages.is_empty());
    assert!(atlas.cache_report().dirty_pages.iter().all(|page| {
        page.dirty_rect.x == 0
            && page.dirty_rect.y == 0
            && page.dirty_rect.width == atlas.plan().atlas_size.x
            && page.dirty_rect.height == atlas.plan().atlas_size.y
    }));

    atlas.mark_prepared_pages_uploaded();
    atlas.prepare(&texts);
    assert!(atlas.cache_report().dirty_pages.is_empty());
}
