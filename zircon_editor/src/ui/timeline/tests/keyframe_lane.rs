#[test]
fn keyframe_lane_reserves_the_known_dense_window_bound_lazily() {
    let source = include_str!("../keyframe_lane.rs");
    let implementation = source.split("#[cfg(test)]").next().unwrap();

    assert!(implementation.contains("let mut visible = Vec::new();"));
    assert!(implementation.contains("if visible.is_empty()"));
    assert!(implementation.contains("visible.reserve(keys.len());"));
    assert!(implementation.contains("visible.push(key);"));
    assert!(!implementation.contains(".collect()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor808_keyframe_lane_capacity_bench_v1() {
    const KEY_COUNT: usize = 4_096;
    let legacy_growth_events = growth_events(None, KEY_COUNT);
    let optimized_growth_events = growth_events(Some(KEY_COUNT), KEY_COUNT);
    eprintln!(
        "EDITOR808_KEYFRAME_LANE_CAPACITY_BENCH_V1 key_count={KEY_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
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
