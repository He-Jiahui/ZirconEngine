use std::hint::black_box;
use std::time::{Duration, Instant};

use super::normalized_promote_document_id;

const PERFORMANCE_MARKER: &str = "EDITOR89_PROMOTION_DOCUMENT_ID_IN_PLACE_TRIM_BENCH_V1";

#[test]
fn optimization_batch_20260826cz_editor89_document_id_trim_preserves_legacy_output() {
    for document_id in [
        "MaterialPreview",
        " Material Preview ",
        "--Material--Preview--",
        "material.preview...",
        "material_preview.v2",
        "\u{00c5}ngstrom.Tool",
        "...",
        "",
    ] {
        assert_eq!(
            normalized_promote_document_id(document_id),
            legacy_normalized_promote_document_id(document_id),
            "{document_id}"
        );
    }
}

#[test]
fn optimization_batch_20260826cz_editor89_document_id_trims_original_buffer_in_place() {
    let source = include_str!("../promotion_state.rs")
        .split_once("#[cfg(test)]")
        .expect("promotion state test boundary should exist")
        .0;
    let normalization = source
        .split_once("pub(super) fn normalized_promote_document_id")
        .expect("document id normalizer should exist")
        .1;

    assert!(normalization.contains("String::with_capacity(trimmed.len())"));
    assert!(normalization.contains("normalized.pop()"));
    assert!(!normalization.contains("trim_matches('.').to_string()"));
}

#[test]
#[ignore = "release-only promotion document id performance gate"]
fn optimization_batch_20260826cz_editor89_document_id_trim_performance_evidence() {
    const DOCUMENT_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR89_PROMOTION_DOCUMENT_ID_IN_PLACE_TRIM_BENCH_V1"
    );
    let document_ids = (0..DOCUMENT_COUNT)
        .map(|index| {
            format!("--Material Preview Runtime Component {index:08} Generated Document--")
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(normalize_batch(
            &document_ids,
            legacy_normalized_promote_document_id,
        ));
        black_box(normalize_batch(
            &document_ids,
            normalized_promote_document_id,
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                normalize_batch(&document_ids, legacy_normalized_promote_document_id)
            }));
            optimized_samples.push(measure(|| {
                normalize_batch(&document_ids, normalized_promote_document_id)
            }));
        } else {
            optimized_samples.push(measure(|| {
                normalize_batch(&document_ids, normalized_promote_document_id)
            }));
            legacy_samples.push(measure(|| {
                normalize_batch(&document_ids, legacy_normalized_promote_document_id)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} documents={DOCUMENT_COUNT} samples={SAMPLE_COUNT} legacy_allocations_per_document=2 optimized_allocations_per_document=1"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place document id trim P95 {optimized_p95_ns}ns must be at most 70% of copied-trim P95 {legacy_p95_ns}ns"
    );
}

fn legacy_normalized_promote_document_id(document_id: &str) -> Option<String> {
    let trimmed = document_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::new();
    let mut previous_was_separator = true;
    for ch in trimmed.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' {
            normalized.push(ch);
            previous_was_separator = false;
        } else if ch.is_ascii_uppercase() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if ch == '.' {
            if !previous_was_separator && !normalized.is_empty() {
                normalized.push('.');
                previous_was_separator = true;
            }
        } else if !previous_was_separator && !normalized.is_empty() {
            normalized.push('.');
            previous_was_separator = true;
        }
    }

    let normalized = normalized.trim_matches('.').to_string();
    (!normalized.is_empty()).then_some(normalized)
}

fn normalize_batch(document_ids: &[String], normalize: fn(&str) -> Option<String>) -> usize {
    document_ids
        .iter()
        .map(|document_id| {
            black_box(normalize(black_box(document_id)))
                .map(|normalized| normalized.len())
                .unwrap_or_default()
        })
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
