use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{remove_style_token_sources, style_token_path_is_at_or_below};

const SAMPLE_PAIRS: usize = 31;
const MATCHES_PER_SAMPLE: usize = 100_000;
const TARGET: &str = "surface.panel.content.background.hovered.token_source";
const CANDIDATES: [&str; 8] = [
    "surface.panel.content.background.hovered.token_source",
    "surface.panel.content.background.hovered.token_source.color",
    "surface.panel.content.background.hovered.token_source[0]",
    "surface.panel.content.background.hovered.token_source_deferred",
    "surface.panel.content.background.hovered.token_sources",
    "surface.panel.content.background.hovered",
    "surface.panel.content.background",
    "other.panel.content.background.hovered.token_source",
];

#[test]
fn optimization_batch_20260829v_runtime295_path_match_preserves_segment_boundaries() {
    let mut sources = BTreeMap::from([
        (TARGET.to_string(), "exact".to_string()),
        (format!("{TARGET}.color"), "nested".to_string()),
        (format!("{TARGET}[0]"), "indexed".to_string()),
        (format!("{TARGET}_deferred"), "sibling".to_string()),
        ("other.path".to_string(), "other".to_string()),
    ]);

    remove_style_token_sources(&mut sources, TARGET);

    assert_eq!(sources.len(), 2);
    assert_eq!(
        sources.get(&format!("{TARGET}_deferred")).unwrap(),
        "sibling"
    );
    assert_eq!(sources.get("other.path").unwrap(), "other");
}

#[test]
fn optimization_batch_20260829v_runtime295_path_match_uses_borrowed_suffixes() {
    let source = include_str!("../tokens.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let remove_body = implementation
        .split("pub(super) fn remove_style_token_sources")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn resolve_value_map").next())
        .expect("style-token removal");

    assert!(implementation.contains("fn style_token_path_is_at_or_below"));
    assert!(remove_body.contains("style_token_path_is_at_or_below(key, path)"));
    assert!(!remove_body.contains("format!(\"{path}.\")"));
    assert!(!remove_body.contains("format!(\"{path}[\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829v_runtime295_allocation_free_style_token_path_match_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME295_ALLOCATION_FREE_STYLE_TOKEN_PATH_MATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
matches_per_sample={MATCHES_PER_SAMPLE} candidate_count={} target_bytes={} \
legacy_prefix_allocations_per_match=2 optimized_prefix_allocations_per_match=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        CANDIDATES.len(),
        TARGET.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_match_count() -> usize {
    let nested = format!("{TARGET}.");
    let indexed = format!("{TARGET}[");
    CANDIDATES
        .iter()
        .filter(|candidate| {
            **candidate == TARGET
                || candidate.starts_with(nested.as_str())
                || candidate.starts_with(indexed.as_str())
        })
        .count()
}

fn optimized_match_count() -> usize {
    CANDIDATES
        .iter()
        .filter(|candidate| style_token_path_is_at_or_below(candidate, TARGET))
        .count()
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..MATCHES_PER_SAMPLE {
        checksum = checksum.wrapping_add(if optimized {
            black_box(optimized_match_count())
        } else {
            black_box(legacy_match_count())
        });
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
