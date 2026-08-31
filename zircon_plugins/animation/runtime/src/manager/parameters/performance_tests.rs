use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};

use super::set_parameter;

const KEY_BYTES: usize = 256;
const UPDATES_PER_SAMPLE: usize = 32_768;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

#[test]
fn set_parameter_updates_existing_keys_inserts_missing_keys_and_rejects_nan() {
    let mut parameters =
        AnimationParameterMap::from([("speed".to_string(), AnimationParameterValue::Scalar(1.0))]);

    set_parameter(
        &mut parameters,
        "speed",
        AnimationParameterValue::Scalar(2.0),
    );
    set_parameter(
        &mut parameters,
        "weight",
        AnimationParameterValue::Scalar(0.5),
    );
    set_parameter(
        &mut parameters,
        "speed",
        AnimationParameterValue::Scalar(f32::NAN),
    );

    assert_eq!(
        parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(2.0))
    );
    assert_eq!(
        parameters.get("weight"),
        Some(&AnimationParameterValue::Scalar(0.5))
    );
}

#[test]
#[ignore = "release-only performance gate"]
fn borrowed_parameter_update_release_benchmark_evidence() {
    let name = "p".repeat(KEY_BYTES);
    let mut legacy_parameters =
        AnimationParameterMap::from([(name.clone(), AnimationParameterValue::Scalar(0.0))]);
    let mut optimized_parameters = legacy_parameters.clone();
    let (legacy_samples, optimized_samples) = paired_samples(
        || legacy_updates(&mut legacy_parameters, &name),
        || optimized_updates(&mut optimized_parameters, &name),
    );
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime170_borrowed_parameter_update key_bytes={KEY_BYTES} updates_per_sample={UPDATES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_key_allocations_per_sample={UPDATES_PER_SAMPLE} optimized_key_allocations_per_sample=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT} legacy_raw_ns={} optimized_raw_ns={}",
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "borrowed parameter update must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn legacy_updates(parameters: &mut AnimationParameterMap, name: &str) {
    for update in 0..UPDATES_PER_SAMPLE {
        parameters.insert(
            name.to_string(),
            AnimationParameterValue::Scalar(update as f32),
        );
    }
    black_box(parameters);
}

fn optimized_updates(parameters: &mut AnimationParameterMap, name: &str) {
    for update in 0..UPDATES_PER_SAMPLE {
        set_parameter(
            parameters,
            name,
            AnimationParameterValue::Scalar(update as f32),
        );
    }
    black_box(parameters);
}

fn paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = ordered.len().saturating_mul(percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
