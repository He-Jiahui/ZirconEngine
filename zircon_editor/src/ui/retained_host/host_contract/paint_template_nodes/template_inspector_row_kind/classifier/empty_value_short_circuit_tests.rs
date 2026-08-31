use std::hint::black_box;
use std::time::Instant;

use super::{inspector_row_kind_from_text, InspectorResourceKind, InspectorRowKind};

const CHECKS_PER_SAMPLE: usize = 1_000_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_inspector_row_kind_from_text(label: &str, value: &str) -> Option<InspectorRowKind> {
    if label.eq_ignore_ascii_case("Lighting") && value.is_empty() {
        return Some(InspectorRowKind::Disclosure);
    }
    if label.eq_ignore_ascii_case("Mesh") && !value.is_empty() {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh));
    }
    if ["Material", "Materials"]
        .iter()
        .any(|candidate| label.eq_ignore_ascii_case(candidate))
        && !value.is_empty()
    {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Material));
    }
    if label.eq_ignore_ascii_case("Cast Shadows") && !value.is_empty() {
        return Some(InspectorRowKind::ShadowSelect);
    }
    if label.eq_ignore_ascii_case("Receive Shadows") && !value.is_empty() {
        return Some(InspectorRowKind::ShadowCheck);
    }
    None
}

fn measure(label: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut kind = None;
    for _ in 0..CHECKS_PER_SAMPLE {
        kind = if optimized {
            inspector_row_kind_from_text(black_box(label), black_box(""))
        } else {
            legacy_inspector_row_kind_from_text(black_box(label), black_box(""))
        };
    }
    black_box(kind);
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
fn optimization_batch_20260829by_editor297_inspector_empty_value_preserves_results() {
    for (label, value) in [
        ("Lighting", ""),
        ("lighting", " "),
        ("Mesh", "mesh.zmesh"),
        ("Material", "material.zmaterial"),
        ("Cast Shadows", "On"),
        ("Receive Shadows", "Off"),
        ("Material", ""),
        ("Unknown", ""),
    ] {
        let value = value.trim();
        assert_eq!(
            inspector_row_kind_from_text(label, value),
            legacy_inspector_row_kind_from_text(label, value),
            "{label:?} {value:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829by_editor297_inspector_empty_value_short_circuits() {
    let source = include_str!("../classifier.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("fn inspector_row_kind_from_text")
        .expect("text classifier")
        .1
        .split_once("fn is_inspector_property_row")
        .expect("row boundary")
        .0;
    assert!(function.contains("if value.is_empty()"));
    assert!(function.contains("then_some(InspectorRowKind::Disclosure)"));
    assert!(!function.contains("&& !value.is_empty()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829by_editor297_empty_value_short_circuit_bench() {
    let label = "XXXXXXXX";
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(label, false));
            candidate.push(measure(label, true));
        } else {
            candidate.push(measure(label, true));
            baseline.push(measure(label, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "EDITOR297_EMPTY_INSPECTOR_VALUE_SHORT_CIRCUIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} baseline_label_branches=5 candidate_label_branches=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
