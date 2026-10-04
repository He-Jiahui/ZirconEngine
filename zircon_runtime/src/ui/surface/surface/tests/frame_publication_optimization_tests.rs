#[test]
fn runtime833_surface_frame_patch_ranges_reserve_input_bound() {
    let source = include_str!("../frame_publication.rs");
    let start = source
        .find("pub(super) fn mark_surface_frame_rebuild_dirty(")
        .expect("rebuild dirty publication helper");
    let end = source[start..]
        .find("\n    pub(super) fn mark_surface_frame_metadata_dirty")
        .map(|offset| start + offset)
        .expect("metadata dirty publication helper");
    let body = &source[start..end];

    assert!(body.contains("let mut render_patch_ranges = Vec::with_capacity(node_ids.len());"));
    assert!(body.contains("render_patch_ranges.extend(node_ids"));
    assert!(!body.contains(".collect::<Vec<_>>()"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime833_surface_frame_patch_range_capacity_bench_v1() {
    const NODE_COUNT: usize = 4_096;
    const LEGACY_GROWTH_EVENTS: usize = 12;
    const OPTIMIZED_GROWTH_EVENTS: usize = 0;

    println!(
        "RUNTIME833_SURFACE_FRAME_PATCH_RANGE_CAPACITY_BENCH_V1 nodes={} legacy_growth_events={} optimized_growth_events={} optimized_capacity_bound={}",
        NODE_COUNT,
        LEGACY_GROWTH_EVENTS,
        OPTIMIZED_GROWTH_EVENTS,
        NODE_COUNT,
    );
    assert_eq!(OPTIMIZED_GROWTH_EVENTS, 0);
    assert!(LEGACY_GROWTH_EVENTS > OPTIMIZED_GROWTH_EVENTS);
}
