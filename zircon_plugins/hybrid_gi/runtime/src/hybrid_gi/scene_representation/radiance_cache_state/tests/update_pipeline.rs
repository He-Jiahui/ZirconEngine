use super::*;

#[test]
fn update_generation_is_hidden_until_filter_border_and_mips_finish() {
    let mut report = HybridGiRadianceCacheUpdateReport::default();
    report.begin(9, 3);
    assert!(!report.generation_is_visible(9));

    report.allocate_trace_tiles();
    report.trace();
    report.filter();
    assert!(!report.generation_is_visible(9));

    report.fixup_borders();
    report.generate_mips();
    assert!(!report.generation_is_visible(9));

    report.complete();
    assert!(report.generation_is_visible(9));
    assert_eq!(report.stage(), HybridGiRadianceCacheUpdateStage::Complete);
    assert_eq!(report.counts(), (3, 3, 3, 3, 3));
}

#[test]
fn update_tracks_marked_demands_when_the_trace_sources_are_all_missing() {
    let mut report = HybridGiRadianceCacheUpdateReport::default();
    let samples = advance_radiance_cache_update_to_mips(&mut report, 12, 5, BTreeMap::new());

    assert!(samples.is_empty());
    assert!(!report.generation_is_visible(12));
    report.complete();
    assert!(report.generation_is_visible(12));
    assert_eq!(report.counts(), (5, 5, 5, 5, 5));
}
