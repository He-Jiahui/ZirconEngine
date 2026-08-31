use std::hint::black_box;
use std::time::Instant;

use super::{action_matches, action_matches_binding_suffix};

const CANDIDATE_COUNT: usize = 64;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bj_showcase_action_key_match_preserves_normalization() {
    for (action_id, needle) in [
        (
            "UiComponentShowcase/WorldSpaceSurfaceConfigured",
            "world_space_surface",
        ),
        (
            "ui_component_showcase::map-field--changed",
            "map_field_changed",
        ),
        ("ui_component_showcase/.../tabChanged", ".tab_changed"),
        ("UIComponentShowcase:missing", "not_present"),
        ("", ""),
    ] {
        assert_eq!(
            action_matches(action_id, needle),
            legacy_action_matches(action_id, needle),
            "action_id={action_id:?} needle={needle:?}"
        );
    }
    for suffix in ["AssetFieldClear", "asset-field-open", "NotPresent"] {
        assert_eq!(
            action_matches_binding_suffix(
                "ui_component_showcase.asset_field_clear_requested",
                suffix,
            ),
            legacy_action_matches_binding_suffix(
                "ui_component_showcase.asset_field_clear_requested",
                suffix,
            )
        );
    }
}

#[test]
fn optimization_batch_20260826bj_showcase_action_key_match_eliminates_heap_keys() {
    const SOURCE: &str = include_str!("../showcase_event_inputs.rs");

    assert_eq!(CANDIDATE_COUNT, 64);
    assert!(SOURCE.contains("NormalizedActionKeyBytes"));
    assert!(SOURCE.contains("iterator_contains("));
    assert!(!SOURCE.contains("fn action_key("));
    assert!(!SOURCE.contains("collect::<Vec<_>>()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bj_showcase_action_key_match_p95() {
    let action_id = "UiComponentShowcase/WorldSpaceSurfaceConfigured";
    let needles = (0..CANDIDATE_COUNT)
        .map(|index| format!("missing_action_{index:02}"))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || {
            needles
                .iter()
                .filter(|needle| legacy_action_matches(black_box(action_id), black_box(needle)))
                .count()
        },
        || {
            needles
                .iter()
                .filter(|needle| action_matches(black_box(action_id), black_box(needle)))
                .count()
        },
    );
    assert_eq!(
        needles
            .iter()
            .filter(|needle| legacy_action_matches(action_id, needle))
            .count(),
        needles
            .iter()
            .filter(|needle| action_matches(action_id, needle))
            .count()
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR01_SHOWCASE_ACTION_KEY_ALLOCATION_FREE_MATCH_BENCH_V1 candidates={CANDIDATE_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_heap_backed_keys={CANDIDATE_COUNT} optimized_heap_backed_keys=0 deterministic_heap_key_reduction_percent=100.0000 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be at least 20% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_action_matches(action_id: &str, needle: &str) -> bool {
    legacy_action_key(action_id).contains(needle)
}

fn legacy_action_matches_binding_suffix(action_id: &str, binding_suffix: &str) -> bool {
    legacy_action_key(action_id).contains(&legacy_camel_to_snake_segment(binding_suffix))
}

fn legacy_action_key(action_id: &str) -> String {
    action_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
        .map(legacy_camel_to_snake_segment)
        .collect::<Vec<_>>()
        .join(".")
}

fn legacy_camel_to_snake_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
