use std::hint::black_box;
use std::time::Instant;

use super::{has_exact_editor_operation_segments, validate_editor_operation};

const MARKER: &str = "RUNTIME240_EDITOR_OPERATION_SEGMENT_VALIDATION_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 262_144;

#[test]
fn optimization_batch_20260826gt_runtime240_editor_operation_requires_three_nonempty_segments() {
    for valid in ["asset.import.run", "Scene.Selection.Clear", "a.b.c"] {
        assert!(has_exact_editor_operation_segments(valid));
        assert!(validate_editor_operation(valid.to_string()).is_ok());
    }
    for invalid in [
        "",
        "asset",
        "asset.import",
        "asset.import.run.now",
        ".import.run",
        "asset..run",
        "asset.import.",
    ] {
        assert!(!has_exact_editor_operation_segments(invalid));
        assert!(validate_editor_operation(invalid.to_string()).is_err());
    }
}

#[test]
fn optimization_batch_20260826gt_runtime240_editor_operation_validation_is_allocation_free() {
    let source = include_str!("../registry.rs");
    assert!(source.contains("let mut segments = value.split('.')"));
    assert!(source.contains("segments.next().is_none()"));
    assert!(!source.contains("value.split('.').collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gt_runtime240_editor_operation_segment_validation_bench() {
    let segment = "EditorOperationSegment".repeat(16);
    let value = format!("{segment}.{segment}.{segment}");
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&value, legacy_has_exact_editor_operation_segments));
            optimized_samples.push(measure(&value, has_exact_editor_operation_segments));
        } else {
            optimized_samples.push(measure(&value, has_exact_editor_operation_segments));
            legacy_samples.push(measure(&value, legacy_has_exact_editor_operation_segments));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "streaming segment validation must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_has_exact_editor_operation_segments(value: &str) -> bool {
    let segments = value.split('.').collect::<Vec<_>>();
    segments.len() == 3 && segments.iter().all(|segment| !segment.is_empty())
}

fn measure(value: &str, implementation: fn(&str) -> bool) -> u64 {
    let started = Instant::now();
    let mut accepted = 0usize;
    for _ in 0..REPEATS {
        accepted += usize::from(implementation(black_box(value)));
    }
    black_box(accepted);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
