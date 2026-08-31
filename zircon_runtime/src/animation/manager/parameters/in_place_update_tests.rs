use std::hint::black_box;
use std::time::Instant;

use super::*;

const PARAMETER_NAME_BYTES: usize = 32 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 512;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hg_runtime253_preserves_parameter_update_semantics() {
    let mut parameters = AnimationParameterMap::from([
        ("speed".to_string(), AnimationParameterValue::Scalar(1.0)),
        ("lives".to_string(), AnimationParameterValue::Integer(3)),
    ]);

    set_parameter(
        &mut parameters,
        "speed",
        AnimationParameterValue::Scalar(2.5),
    );
    set_parameter(
        &mut parameters,
        "grounded",
        AnimationParameterValue::Bool(true),
    );
    assert_eq!(
        parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(2.5))
    );
    assert_eq!(
        parameters.get("grounded"),
        Some(&AnimationParameterValue::Bool(true))
    );

    set_parameter(
        &mut parameters,
        "speed",
        AnimationParameterValue::Scalar(f32::NAN),
    );
    set_parameter(
        &mut parameters,
        "invalid",
        AnimationParameterValue::Vec2([f32::INFINITY, 0.0]),
    );
    assert_eq!(
        parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(2.5))
    );
    assert!(!parameters.contains_key("invalid"));
}

#[test]
fn optimization_batch_20260826hg_runtime253_updates_existing_parameter_in_place() {
    let source = include_str!("../parameters.rs");
    let start = source
        .find("pub(super) fn set_parameter(")
        .expect("set_parameter function");
    let end = source[start..]
        .find("\npub(super) fn numeric_parameter")
        .map(|offset| start + offset)
        .expect("next function boundary");
    let body = &source[start..end];

    assert!(body.contains("parameters.get_mut(name)"));
    assert!(body.contains("*current = value"));
    assert!(body.contains("parameters.insert(name.to_string(), value)"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hg_runtime253_animation_parameter_in_place_release_benchmark() {
    let name = "parameter".repeat(PARAMETER_NAME_BYTES / "parameter".len());
    let baseline =
        AnimationParameterMap::from([(name.clone(), AnimationParameterValue::Scalar(0.0))]);
    let mut legacy = baseline.clone();
    let mut optimized = baseline;

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for operation in 0..OPERATIONS_PER_SAMPLE {
                legacy_set_parameter(
                    black_box(&mut legacy),
                    black_box(&name),
                    AnimationParameterValue::Scalar(operation as f32),
                );
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for operation in 0..OPERATIONS_PER_SAMPLE {
                set_parameter(
                    black_box(&mut optimized),
                    black_box(&name),
                    AnimationParameterValue::Scalar(operation as f32),
                );
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME253_ANIMATION_PARAMETER_IN_PLACE_BENCH_V1 \
         parameter_name_bytes={PARAMETER_NAME_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_set_parameter(
    parameters: &mut AnimationParameterMap,
    name: &str,
    value: AnimationParameterValue,
) {
    if animation_parameter_value_is_finite(&value) {
        parameters.insert(name.to_string(), value);
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
