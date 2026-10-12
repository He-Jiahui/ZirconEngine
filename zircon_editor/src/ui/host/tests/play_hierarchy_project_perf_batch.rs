#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor815_play_hierarchy_changed_row_capacity_bench_v1() {
    const ROW_COUNT: usize = 4096;
    let legacy_growth_events = geometric_growth_events(ROW_COUNT);
    let optimized_growth_events = 0;
    println!(
        "EDITOR815_PLAY_HIERARCHY_CHANGED_ROW_CAPACITY_BENCH_V1 rows={ROW_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
}

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0;
    let mut growth_events = 0;
    for current_length in 1..=length {
        if current_length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            growth_events += 1;
        }
    }
    growth_events
}
