use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::{
    UiResourceFallbackMode, UiResourceFallbackPolicy, UiResourceKind,
};

use super::*;
use crate::ui::template::asset::resource_ref::UiResourceResolveDiagnosticCode;

const BENCHMARK_MARKER: &str = "RUNTIME64_UI_RESOURCE_CACHED_DIAGNOSTIC_INDEX_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const SCANS_PER_SAMPLE: usize = 4;
const DIAGNOSTIC_COUNT: usize = 4_096;
const LOOKUP_COUNT: usize = 512;

fn diagnostic(index: usize) -> UiResourceResolveDiagnostic {
    UiResourceResolveDiagnostic {
        code: UiResourceResolveDiagnosticCode::MissingPrimary,
        severity: UiResourceDiagnosticSeverity::Warning,
        uri: format!("res://textures/resource_{index}.png"),
        message: format!("missing resource {index}"),
    }
}

fn reference(primary: usize, fallback: Option<usize>) -> UiResourceRef {
    UiResourceRef {
        kind: UiResourceKind::Image,
        uri: format!("res://textures/resource_{primary}.png"),
        fallback: UiResourceFallbackPolicy {
            mode: if fallback.is_some() {
                UiResourceFallbackMode::Placeholder
            } else {
                UiResourceFallbackMode::None
            },
            uri: fallback.map(|index| format!("res://textures/resource_{index}.png")),
        },
    }
}

fn legacy_cached_indices(
    reference: &UiResourceRef,
    diagnostic_index: usize,
    diagnostics: &[UiResourceResolveDiagnostic],
) -> Vec<usize> {
    let mut indices = diagnostics
        .iter()
        .enumerate()
        .filter_map(|(index, diagnostic)| {
            if diagnostic_matches_reference(diagnostic, reference) {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if indices.is_empty() {
        indices.push(diagnostic_index);
    }
    indices
}

fn fixtures() -> (Vec<UiResourceResolveDiagnostic>, Vec<UiResourceRef>) {
    let diagnostics = (0..DIAGNOSTIC_COUNT).map(diagnostic).collect();
    let references = (0..LOOKUP_COUNT)
        .map(|index| reference(index * 2, Some(index * 2 + 1)))
        .collect();
    (diagnostics, references)
}

fn legacy_scan(diagnostics: &[UiResourceResolveDiagnostic], references: &[UiResourceRef]) -> usize {
    references
        .iter()
        .map(|reference| legacy_cached_indices(reference, 0, diagnostics).len())
        .sum()
}

fn optimized_scan(
    diagnostics: &[UiResourceResolveDiagnostic],
    references: &[UiResourceRef],
) -> usize {
    let diagnostic_indices_by_uri = diagnostic_index_by_uri(diagnostics);
    references
        .iter()
        .map(|reference| {
            diagnostic_indices_for_cached_resolution(reference, 0, &diagnostic_indices_by_uri).len()
        })
        .sum()
}

fn sample_ns(mut scan: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut observed = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        observed += black_box(scan());
    }
    black_box(observed);
    started.elapsed().as_nanos()
}

fn percentile(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826bc_cached_diagnostic_index_preserves_global_order() {
    let diagnostics = vec![diagnostic(3), diagnostic(99), diagnostic(2), diagnostic(3)];
    let reference = reference(2, Some(3));
    let index = diagnostic_index_by_uri(&diagnostics);
    assert_eq!(
        diagnostic_indices_for_cached_resolution(&reference, 7, &index),
        vec![0, 2, 3]
    );

    let missing_reference = reference(200, None);
    assert_eq!(
        diagnostic_indices_for_cached_resolution(&missing_reference, 7, &index),
        vec![7]
    );
    let same_uri = reference(3, Some(3));
    assert_eq!(
        diagnostic_indices_for_cached_resolution(&same_uri, 7, &index),
        vec![0, 3]
    );
}

#[test]
fn optimization_batch_20260826bc_cached_diagnostic_resolution_uses_uri_index() {
    let source = include_str!("../resolution_report.rs");

    assert!(source.contains("HashMap<String, Vec<usize>>"));
    assert!(source.contains("diagnostic_indices_by_uri"));
    assert!(source.contains("indices.sort_unstable();"));
    assert!(!source.contains("diagnostics\n                .iter()\n                .enumerate()"));
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260826bc_cached_diagnostic_index_p95() {
    let (diagnostics, references) = fixtures();
    assert_eq!(
        legacy_scan(&diagnostics, &references),
        optimized_scan(&diagnostics, &references)
    );
    for _ in 0..3 {
        black_box(legacy_scan(&diagnostics, &references));
        black_box(optimized_scan(&diagnostics, &references));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(|| legacy_scan(&diagnostics, &references)));
            optimized_samples.push(sample_ns(|| optimized_scan(&diagnostics, &references)));
        } else {
            optimized_samples.push(sample_ns(|| optimized_scan(&diagnostics, &references)));
            legacy_samples.push(sample_ns(|| legacy_scan(&diagnostics, &references)));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples.clone(), 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction = 100.0 - (optimized_p95 as f64 * 100.0 / legacy_p95 as f64);
    println!(
        "{BENCHMARK_MARKER} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} reduction_percent={reduction:.3} diagnostics={DIAGNOSTIC_COUNT} lookups={LOOKUP_COUNT} scans_per_sample={SCANS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS}"
    );

    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(20),
        "expected cached diagnostic index P95 to be at least 80% below repeated diagnostic scans; legacy={legacy_p95}ns optimized={optimized_p95}ns reduction={reduction:.3}%"
    );
}
