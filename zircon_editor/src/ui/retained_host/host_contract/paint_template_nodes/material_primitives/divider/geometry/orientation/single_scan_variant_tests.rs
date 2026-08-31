use std::hint::black_box;
use std::time::Instant;

use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::divider_is_vertical;

const CHECKS_PER_SAMPLE: usize = 4096;
const SAMPLE_PAIRS: usize = 31;
const VARIANT_BYTES: usize = 2048;

fn legacy_contains(value: &str, expected: &str) -> bool {
    value
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.eq_ignore_ascii_case(expected))
}

fn legacy_is_vertical(value: &str, rect: &FrameRect) -> bool {
    legacy_contains(value, "vertical")
        || legacy_contains(value, "wrapperVertical")
        || (!legacy_contains(value, "horizontal") && rect.height > rect.width * 1.4)
}

fn measure(value: &str, rect: &FrameRect, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut vertical = false;
    let mut node = TemplatePaneNodeData::default();
    node.component_variant = value.to_owned();
    for _ in 0..CHECKS_PER_SAMPLE {
        vertical = if optimized {
            divider_is_vertical(black_box(&node), black_box(rect))
        } else {
            legacy_is_vertical(black_box(value), black_box(rect))
        };
    }
    black_box(vertical);
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

#[test]
fn optimization_batch_20260829bo_editor287_divider_orientation_preserves_results() {
    let wide = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 20.0,
    };
    let tall = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 20.0,
        height: 100.0,
    };
    for (value, rect) in [
        ("vertical", &wide),
        ("wrapperVertical", &wide),
        ("horizontal", &tall),
        ("", &tall),
        ("\u{4f8b}", &wide),
    ] {
        let mut node = TemplatePaneNodeData::default();
        node.component_variant = value.to_owned();
        assert_eq!(
            divider_is_vertical(&node, rect),
            legacy_is_vertical(value, rect),
            "{value:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bo_editor287_divider_orientation_uses_one_scan() {
    let source = include_str!("../orientation.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains("for part in component_variant.split"));
    assert!(production.contains("has_vertical"));
    assert!(production.contains("has_horizontal"));
    assert!(!production.contains("component_variant_contains"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bo_editor287_single_scan_divider_orientation_bench() {
    let value = "x".repeat(VARIANT_BYTES);
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 20.0,
        height: 100.0,
    };
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&value, &rect, false));
            candidate.push(measure(&value, &rect, true));
        } else {
            candidate.push(measure(&value, &rect, true));
            baseline.push(measure(&value, &rect, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "EDITOR287_SINGLE_SCAN_DIVIDER_ORIENTATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} variant_bytes={VARIANT_BYTES} baseline_variant_scans=3 candidate_variant_scans=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
