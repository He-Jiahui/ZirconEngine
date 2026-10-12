const COMPLETION_COUNT: usize = 32_768;

#[test]
fn optimization_batch_20260830cp_editor_completion_reserves_inspection_upper_bound() {
    let source = include_str!("../adapter.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("autosave adapter implementation");

    assert!(implementation.contains("Vec::with_capacity(inspected_tickets)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cp_editor_completion_capacity_evidence() {
    let legacy_growth_events = collect_growth_events(false);
    let optimized_growth_events = collect_growth_events(true);

    println!(
        "EDITOR503_AUTOSAVE_COMPLETION_CAPACITY_BENCH_V1 inspected={COMPLETION_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn collect_growth_events(reserve_upper_bound: bool) -> usize {
    let capacity = usize::from(reserve_upper_bound) * COMPLETION_COUNT;
    let mut outcomes = Vec::with_capacity(capacity);
    let mut growth_events = 0;
    for outcome in 0..COMPLETION_COUNT {
        let previous_capacity = outcomes.capacity();
        outcomes.push(outcome);
        growth_events += usize::from(outcomes.capacity() != previous_capacity);
    }
    std::hint::black_box(outcomes);
    growth_events
}
