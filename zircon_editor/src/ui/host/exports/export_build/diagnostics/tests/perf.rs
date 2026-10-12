use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const DIAGNOSTIC_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_is_editor629_reserves_export_diagnostic_membership() {
    let source = include_str!("../../diagnostics.rs");
    let normalize_body = source
        .split("fn normalize_export_diagnostics")
        .nth(1)
        .expect("diagnostic normalizer remains present")
        .split("pub(super) fn cargo_invocation_diagnostics")
        .next()
        .expect("diagnostic normalizer remains bounded");

    assert!(normalize_body.contains("HashSet::with_capacity(diagnostics.len())"));
    assert!(!normalize_body.contains("let mut seen = HashSet::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_is_editor629_preallocated_export_diagnostic_benchmark() {
    let diagnostics = (0..DIAGNOSTIC_COUNT)
        .map(|index| format!(" export diagnostic {index:08} "))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_diagnostic_dedupe(&diagnostics, false));
        black_box(measure_diagnostic_dedupe(&diagnostics, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_diagnostic_dedupe(&diagnostics, false));
            preallocated_samples.push(measure_diagnostic_dedupe(&diagnostics, true));
        } else {
            preallocated_samples.push(measure_diagnostic_dedupe(&diagnostics, true));
            unreserved_samples.push(measure_diagnostic_dedupe(&diagnostics, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR629_PREALLOCATED_EXPORT_DIAGNOSTIC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} diagnostic_count={DIAGNOSTIC_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=15",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 85 / 100);
}

fn measure_diagnostic_dedupe(diagnostics: &[String], preallocated: bool) -> u128 {
    let mut seen = if preallocated {
        HashSet::with_capacity(diagnostics.len())
    } else {
        HashSet::new()
    };
    let started = Instant::now();
    for diagnostic in diagnostics {
        black_box(seen.insert(black_box(diagnostic.trim().to_string())));
    }
    black_box(seen);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
