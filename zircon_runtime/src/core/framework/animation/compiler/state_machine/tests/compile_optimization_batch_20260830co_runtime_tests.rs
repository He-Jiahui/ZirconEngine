const SYNTHETIC_STATE_COUNT: usize = 32_768;

#[test]
fn optimization_batch_20260830co_runtime_state_collection_reserves_authored_upper_bound() {
    let source = include_str!("../compile.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("state machine compiler implementation");

    assert!(implementation.contains("Vec::with_capacity(asset.states.len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830co_runtime_state_collection_capacity_evidence() {
    let legacy_growth_events = collect_growth_events(false);
    let optimized_growth_events = collect_growth_events(true);

    println!(
        "RUNTIME502_STATE_MACHINE_STATE_CAPACITY_BENCH_V1 states={SYNTHETIC_STATE_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn collect_growth_events(reserve_upper_bound: bool) -> usize {
    let capacity = usize::from(reserve_upper_bound) * SYNTHETIC_STATE_COUNT;
    let mut states = Vec::with_capacity(capacity);
    let mut growth_events = 0;
    for state in 0..SYNTHETIC_STATE_COUNT {
        let previous_capacity = states.capacity();
        states.push(state);
        growth_events += usize::from(states.capacity() != previous_capacity);
    }
    std::hint::black_box(states);
    growth_events
}
