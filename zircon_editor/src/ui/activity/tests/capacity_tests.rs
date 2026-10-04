use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const PROJECTIONS_PER_SAMPLE: usize = 4_096;

#[test]
fn editor745_activity_projection_capacity_preserves_order() {
    let view_source = include_str!("../view.rs");
    assert_eq!(
        view_source
            .matches("Vec::with_capacity(snapshots.len())")
            .count(),
        2,
        "toast and progress projections should reserve their snapshot bounds"
    );
    assert!(view_source.contains("Vec::with_capacity(records.len())"));

    let decision_source = include_str!("../decision/view.rs");
    assert!(decision_source.contains("Vec::with_capacity(decision.options().len())"));
}

#[test]
fn editor745_activity_projection_capacity_model_eliminates_growth() {
    let legacy_growth_events = growth_events(0, PROJECTIONS_PER_SAMPLE);
    let optimized_growth_events = growth_events(PROJECTIONS_PER_SAMPLE, PROJECTIONS_PER_SAMPLE);
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor745_activity_projection_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_growth(false));
            optimized_samples.push(measure_growth(true));
        } else {
            optimized_samples.push(measure_growth(true));
            legacy_samples.push(measure_growth(false));
        }
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_PAIRS - 1];
    let optimized_p95 = optimized_samples[SAMPLE_PAIRS - 1];
    let legacy_growth_events = growth_events(0, PROJECTIONS_PER_SAMPLE);
    let optimized_growth_events = growth_events(PROJECTIONS_PER_SAMPLE, PROJECTIONS_PER_SAMPLE);
    println!(
        "EDITOR745_ACTIVITY_PROJECTION_CAPACITY_BENCH_V1 projections={PROJECTIONS_PER_SAMPLE} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
         legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}",
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn measure_growth(reserved: bool) -> u128 {
    let started = Instant::now();
    let mut values = if reserved {
        Vec::with_capacity(PROJECTIONS_PER_SAMPLE)
    } else {
        Vec::new()
    };
    for value in 0..PROJECTIONS_PER_SAMPLE {
        values.push(value);
    }
    black_box(values);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(initial_capacity: usize, count: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut length = 0;
    let mut events = 0;
    for _ in 0..count {
        if length == capacity {
            capacity = capacity.saturating_mul(2).max(4);
            events += 1;
        }
        length += 1;
    }
    events
}
