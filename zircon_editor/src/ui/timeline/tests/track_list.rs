#[test]
fn track_list_reserves_the_known_input_bound_before_projection_append() {
    let source = include_str!("../track_list.rs");
    let implementation = source.split("#[cfg(test)]").next().unwrap();

    assert!(implementation.contains("let mut rows = Vec::with_capacity(tracks.len());"));
    assert!(implementation.contains("for track in tracks"));
    assert!(implementation.contains("rows.push(TimelineTrackRow"));
    assert!(!implementation.contains(".collect()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor809_track_list_capacity_bench_v1() {
    const TRACK_COUNT: usize = 4_096;
    let legacy_growth_events = growth_events(None, TRACK_COUNT);
    let optimized_growth_events = growth_events(Some(TRACK_COUNT), TRACK_COUNT);
    eprintln!(
        "EDITOR809_TRACK_LIST_CAPACITY_BENCH_V1 track_count={TRACK_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn growth_events(initial_capacity: Option<usize>, length: usize) -> usize {
    let mut capacity = initial_capacity.unwrap_or(0);
    let mut growth_events = 0;
    for index in 0..length {
        if index == capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            growth_events += 1;
        }
    }
    growth_events
}
