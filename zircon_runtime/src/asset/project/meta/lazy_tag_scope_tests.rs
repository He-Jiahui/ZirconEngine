use std::hint::black_box;
use std::time::Instant;

use super::{validate_tag_list, AssetMetaError, EntryTagScope};

const SAMPLE_PAIRS: usize = 31;
const ENTRY_CHECKS_PER_SAMPLE: usize = 32_768;

#[test]
fn optimization_batch_20260828io_runtime287_lazy_scope_preserves_entry_error_text() {
    let tags = vec!["valid".to_string(), " duplicate ".to_string()];

    let error = validate_tag_list(EntryTagScope(7), &tags)
        .expect_err("surrounding whitespace should remain invalid");

    assert_eq!(
        error,
        AssetMetaError::TagHasSurroundingWhitespace {
            scope: "entries[7]".to_string(),
            tag: " duplicate ".to_string(),
        }
    );
}

#[test]
fn optimization_batch_20260828io_runtime287_entry_scope_is_formatted_only_on_error() {
    let source = include_str!("../meta.rs");

    assert!(source.contains("struct EntryTagScope(usize);"));
    assert!(source.contains("validate_tag_list(EntryTagScope(index), &entry.tags)"));
    assert!(source.contains("validate_tag_set(EntryTagScope(index), &entry.tags)"));
    assert!(source.contains("validate_tag_value(EntryTagScope(index), entry.get(\"tags\"))"));
    assert!(!source.contains("&format!(\"entries[{index}]\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828io_runtime287_lazy_meta_tag_scope_bench() {
    let tags = Vec::<String>::new();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&tags));
            optimized_samples.push(measure_optimized(&tags));
        } else {
            optimized_samples.push(measure_optimized(&tags));
            legacy_samples.push(measure_legacy(&tags));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME287_LAZY_META_TAG_SCOPE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
entry_checks_per_sample={ENTRY_CHECKS_PER_SAMPLE} legacy_scope_allocations_per_sample={ENTRY_CHECKS_PER_SAMPLE} \
optimized_scope_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure_legacy(tags: &[String]) -> u128 {
    let started = Instant::now();
    for index in 0..ENTRY_CHECKS_PER_SAMPLE {
        let scope = format!("entries[{index}]");
        black_box(validate_tag_list(
            black_box(scope.as_str()),
            black_box(tags),
        ))
        .unwrap();
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(tags: &[String]) -> u128 {
    let started = Instant::now();
    for index in 0..ENTRY_CHECKS_PER_SAMPLE {
        black_box(validate_tag_list(
            black_box(EntryTagScope(index)),
            black_box(tags),
        ))
        .unwrap();
    }
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
