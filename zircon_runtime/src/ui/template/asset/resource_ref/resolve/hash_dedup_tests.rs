use std::hint::black_box;
use std::time::Instant;

use super::{dedup_resource_diagnostics, UiResourceDiagnostic, UiResourceDiagnosticSeverity};

const MARKER: &str = "RUNTIME246_RESOURCE_DIAGNOSTIC_HASH_DEDUP_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const DIAGNOSTIC_COUNT: usize = 2_048;
const UNIQUE_DIAGNOSTICS: usize = 16;
const REPEATS: usize = 32;

#[test]
fn optimization_batch_20260826gz_runtime246_resource_diagnostics_stay_unique_and_sorted() {
    let mut diagnostics = vec![
        diagnostic(2, UiResourceDiagnosticSeverity::Warning),
        diagnostic(1, UiResourceDiagnosticSeverity::Warning),
        diagnostic(1, UiResourceDiagnosticSeverity::Error),
        diagnostic(3, UiResourceDiagnosticSeverity::Error),
    ];

    dedup_resource_diagnostics(&mut diagnostics);

    assert_eq!(
        diagnostics
            .iter()
            .map(diagnostic_identity)
            .collect::<Vec<_>>(),
        [
            ("node.0001", "resource.0001", "diagnostic message 0001"),
            ("node.0002", "resource.0002", "diagnostic message 0002"),
            ("node.0003", "resource.0003", "diagnostic message 0003"),
        ]
    );
}

#[test]
fn optimization_batch_20260826gz_runtime246_resource_diagnostics_hash_before_sorting() {
    let source = include_str!("../resolve.rs");
    let implementation = source
        .split("fn dedup_resource_diagnostics")
        .nth(1)
        .and_then(|tail| tail.split("fn validate_uri").next())
        .expect("resource diagnostic dedup implementation");
    assert!(implementation.contains("HASH_DEDUP_DIAGNOSTIC_THRESHOLD"));
    assert!(implementation.contains("HashSet::with_capacity"));
    assert!(implementation.contains("diagnostics.retain"));
    assert!(implementation.contains("drop(seen)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gz_runtime246_resource_diagnostic_hash_dedup_bench() {
    let diagnostics = (0..DIAGNOSTIC_COUNT)
        .map(|index| {
            diagnostic(
                index % UNIQUE_DIAGNOSTICS,
                UiResourceDiagnosticSeverity::Warning,
            )
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let mut legacy_workloads = workloads(&diagnostics);
        let mut optimized_workloads = workloads(&diagnostics);
        if pair % 2 == 0 {
            legacy_samples.push(measure(
                &mut legacy_workloads,
                legacy_dedup_resource_diagnostics,
            ));
            optimized_samples.push(measure(
                &mut optimized_workloads,
                dedup_resource_diagnostics,
            ));
        } else {
            optimized_samples.push(measure(
                &mut optimized_workloads,
                dedup_resource_diagnostics,
            ));
            legacy_samples.push(measure(
                &mut legacy_workloads,
                legacy_dedup_resource_diagnostics,
            ));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "hash deduplication must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn diagnostic(index: usize, severity: UiResourceDiagnosticSeverity) -> UiResourceDiagnostic {
    UiResourceDiagnostic {
        code: format!("resource.{index:04}"),
        severity,
        message: format!("diagnostic message {index:04}"),
        path: format!("node.{index:04}"),
    }
}

fn diagnostic_identity(diagnostic: &UiResourceDiagnostic) -> (&str, &str, &str) {
    (
        diagnostic.path.as_str(),
        diagnostic.code.as_str(),
        diagnostic.message.as_str(),
    )
}

fn legacy_dedup_resource_diagnostics(diagnostics: &mut Vec<UiResourceDiagnostic>) {
    diagnostics.sort_by(|left, right| {
        (&left.path, &left.code, &left.message).cmp(&(&right.path, &right.code, &right.message))
    });
    diagnostics.dedup_by(|left, right| {
        left.path == right.path && left.code == right.code && left.message == right.message
    });
}

fn workloads(diagnostics: &[UiResourceDiagnostic]) -> Vec<Vec<UiResourceDiagnostic>> {
    (0..REPEATS).map(|_| diagnostics.to_vec()).collect()
}

fn measure(
    workloads: &mut [Vec<UiResourceDiagnostic>],
    implementation: fn(&mut Vec<UiResourceDiagnostic>),
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for diagnostics in black_box(workloads) {
        implementation(diagnostics);
        checksum = checksum.wrapping_add(diagnostics.len());
        black_box(&diagnostics);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
