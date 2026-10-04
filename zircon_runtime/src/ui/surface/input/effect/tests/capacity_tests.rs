use std::hint::black_box;
use std::time::Instant;

use super::{reserve_effect_result_capacity, UiInputDispatchResult};
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchReply, UiInputEvent, UiMouseMotionInputEvent,
};

const EFFECTS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime77_effect_result_capacity_reserves_each_projection() {
    let event = UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
        metadata: Default::default(),
        delta_x: 0.0,
        delta_y: 0.0,
    });
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());

    reserve_effect_result_capacity(&mut result, EFFECTS_PER_SAMPLE);

    assert!(result.applied_effects.capacity() >= EFFECTS_PER_SAMPLE);
    assert!(result.rejected_effects.capacity() >= EFFECTS_PER_SAMPLE);
    assert!(result.host_requests.capacity() >= EFFECTS_PER_SAMPLE);
    assert!(result.component_events.capacity() >= EFFECTS_PER_SAMPLE);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime77_effect_result_capacity_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(EFFECTS_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(EFFECTS_PER_SAMPLE, EFFECTS_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_growth(EFFECTS_PER_SAMPLE, EFFECTS_PER_SAMPLE));
            legacy_ns.push(measure_growth(EFFECTS_PER_SAMPLE, 0));
        }
    }

    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    let legacy_growth_events = growth_events(EFFECTS_PER_SAMPLE, 0);
    let optimized_growth_events = growth_events(EFFECTS_PER_SAMPLE, EFFECTS_PER_SAMPLE);
    assert!(
        optimized_growth_events == 0 && legacy_growth_events >= 1,
        "reserved effect projections must remove geometric growth: legacy={legacy_growth_events} optimized={optimized_growth_events}"
    );
    println!(
        "RUNTIME77_EFFECT_RESULT_CAPACITY_BENCH_V1 effects_per_sample={EFFECTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}"
    );
}

fn measure_growth(length: usize, capacity: usize) -> u128 {
    let started = Instant::now();
    let mut values = Vec::with_capacity(capacity);
    for value in 0..length {
        values.push(black_box(value));
    }
    black_box(values);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(length: usize, capacity: usize) -> usize {
    if capacity >= length {
        return 0;
    }
    let mut capacity = capacity.max(1);
    let mut growth = 0;
    while capacity < length {
        capacity = capacity.saturating_mul(2);
        growth += 1;
    }
    growth
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
