use std::hint::black_box;
use std::time::Instant;

use super::*;

const PROPERTY: &str = "value_percent";
const SAMPLE_PAIRS: usize = 17;
const UPDATES_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_fp_runtime472_reuses_existing_slider_value_key() {
    let mut state = UiComponentState::new();
    state
        .values
        .insert(PROPERTY.to_owned(), UiValue::Float(0.25));

    set_slider_value(&mut state, PROPERTY, UiValue::Float(0.75));

    assert_eq!(state.values.len(), 1);
    assert_eq!(state.values.get(PROPERTY), Some(&UiValue::Float(0.75)));

    let mut missing = UiComponentState::new();
    set_slider_value(&mut missing, PROPERTY, UiValue::Float(0.5));
    assert_eq!(missing.values.get(PROPERTY), Some(&UiValue::Float(0.5)));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fp_runtime472_borrowed_slider_value_key_benchmark() {
    for _ in 0..4 {
        black_box(measure_existing_key(false));
        black_box(measure_existing_key(true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_existing_key(false));
            optimized_samples.push(measure_existing_key(true));
        } else {
            optimized_samples.push(measure_existing_key(true));
            legacy_samples.push(measure_existing_key(false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME472_BORROWED_SLIDER_VALUE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} updates_per_sample={UPDATES_PER_SAMPLE} legacy_owned_keys_per_sample={UPDATES_PER_SAMPLE} optimized_owned_keys_per_sample=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 70 / 100);
}

fn measure_existing_key(optimized: bool) -> u128 {
    let mut state = UiComponentState::new();
    state
        .values
        .insert(PROPERTY.to_owned(), UiValue::Float(0.0));
    let started = Instant::now();
    for update in 0..UPDATES_PER_SAMPLE {
        let value = UiValue::Float((update & 1_023) as f64 / 1_023.0);
        if optimized {
            set_slider_value(black_box(&mut state), PROPERTY, value);
        } else {
            legacy_set_slider_value(black_box(&mut state), PROPERTY, value);
        }
    }
    black_box(state);
    started.elapsed().as_nanos().max(1)
}

fn legacy_set_slider_value(state: &mut UiComponentState, property: &str, value: UiValue) {
    state.reference_sources.remove(property);
    state.values.insert(property.to_owned(), value);
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
