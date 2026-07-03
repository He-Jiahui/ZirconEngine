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
