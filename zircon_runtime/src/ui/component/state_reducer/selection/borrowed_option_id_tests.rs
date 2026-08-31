use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::enum_option_id_matches;

const SAMPLE_PAIRS: usize = 21;
const SCANS_PER_SAMPLE: usize = 8_192;
const VALUE_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826ed_runtime173_option_match_preserves_enum_only_semantics() {
    assert!(enum_option_id_matches(
        &UiValue::Enum("selected-option".to_string()),
        "selected-option"
    ));
    assert!(!enum_option_id_matches(
        &UiValue::Enum("another-option".to_string()),
        "selected-option"
    ));
    assert!(!enum_option_id_matches(
        &UiValue::String("selected-option".to_string()),
        "selected-option"
    ));
}

#[test]
fn optimization_batch_20260826ed_runtime173_option_match_uses_borrowed_comparison() {
    let source = include_str!("../selection.rs");
    let helper_start = source.find("fn enum_option_id_matches").unwrap();
    let helper_end = source[helper_start..]
        .find("fn option_is_disabled")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];
    assert!(!helper_source.contains("clone()"));
    assert!(!source.contains("UiValue::Enum(option_id.clone())"));
    assert_eq!(
        source
            .matches("enum_option_id_matches(value, &option_id)")
            .count(),
        2
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ed_runtime173_selection_borrowed_option_id_bench() {
    let values = fixture_values();
    let option_id = "selected-option-with-a-stable-long-identifier";
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&values, option_id));
            optimized_samples.push(measure_optimized(&values, option_id));
        } else {
            optimized_samples.push(measure_optimized(&values, option_id));
            legacy_samples.push(measure_legacy(&values, option_id));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME173_SELECTION_BORROWED_OPTION_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
scans_per_sample={SCANS_PER_SAMPLE} values_per_scan={VALUE_COUNT} \
legacy_allocations_per_compared_enum=1 optimized_allocations_per_compared_enum=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed option-id scan P95 {optimized_p95_ns}ns must be at most 70% of allocating scan P95 {legacy_p95_ns}ns"
    );
}

fn fixture_values() -> Vec<UiValue> {
    let mut values = (0..VALUE_COUNT)
        .map(|index| UiValue::Enum(format!("option-{index}")))
        .collect::<Vec<_>>();
    values[VALUE_COUNT - 1] =
        UiValue::Enum("selected-option-with-a-stable-long-identifier".to_string());
    values
}

fn legacy_matches(value: &UiValue, option_id: &str) -> bool {
    value == &UiValue::Enum(option_id.to_string())
}

fn measure_legacy(values: &[UiValue], option_id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        checksum ^= black_box(values)
            .iter()
            .any(|value| legacy_matches(black_box(value), black_box(option_id)))
            as usize;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(values: &[UiValue], option_id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        checksum ^= black_box(values)
            .iter()
            .any(|value| enum_option_id_matches(black_box(value), black_box(option_id)))
            as usize;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
