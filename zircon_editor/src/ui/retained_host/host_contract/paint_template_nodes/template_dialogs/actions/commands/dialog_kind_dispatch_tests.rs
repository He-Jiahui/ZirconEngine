use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy)]
enum DialogKind {
    Confirm,
    Alert,
    Single,
}

fn legacy_kind_score(kind: DialogKind) -> usize {
    if matches!(kind, DialogKind::Confirm) {
        1
    } else if matches!(kind, DialogKind::Alert) {
        2
    } else {
        3
    }
}

fn optimized_kind_score(kind: DialogKind) -> usize {
    match kind {
        DialogKind::Confirm => 1,
        DialogKind::Alert => 2,
        DialogKind::Single => 3,
    }
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..1_000_000 {
        let kind = black_box(DialogKind::Single);
        evidence += if optimized {
            optimized_kind_score(kind)
        } else {
            legacy_kind_score(kind)
        };
    }
    black_box(evidence);
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
fn optimization_batch_20260830cb_editor300_dialog_kind_dispatch_preserves_results() {
    for kind in [DialogKind::Confirm, DialogKind::Alert, DialogKind::Single] {
        assert_eq!(optimized_kind_score(kind), legacy_kind_score(kind));
    }
}

#[test]
fn optimization_batch_20260830cb_editor300_production_uses_single_kind_match() {
    let source = include_str!("../commands.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert_eq!(production.matches("match kind").count(), 1);
    assert_eq!(production.matches("matches!(kind").count(), 0);
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260830cb_editor300_dialog_kind_dispatch_benchmark() {
    let mut baseline = Vec::with_capacity(31);
    let mut candidate = Vec::with_capacity(31);
    for pair in 0..31 {
        if pair % 2 == 0 {
            baseline.push(measure(false));
            candidate.push(measure(true));
        } else {
            candidate.push(measure(true));
            baseline.push(measure(false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "EDITOR300_DIALOG_KIND_DISPATCH_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
