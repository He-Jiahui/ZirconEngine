use std::hint::black_box;
use std::time::Instant;

use super::single_buffer_profiling_status;

const SAMPLE_PAIRS: usize = 31;
const MESSAGES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829af_editor251_profiling_status_preserves_text() {
    for (editor, runtime, expected) in [
        (
            "capture started",
            "capture forwarded",
            "Editor profiling: capture started; Runtime profiling: capture forwarded",
        ),
        (
            "capture stopped",
            "unavailable",
            "Editor profiling: capture stopped; Runtime profiling: unavailable",
        ),
        (
            "capture failed",
            "runtime channel closed",
            "Editor profiling: capture failed; Runtime profiling: runtime channel closed",
        ),
    ] {
        assert_eq!(single_buffer_profiling_status(editor, runtime), expected);
        assert_eq!(
            single_buffer_profiling_status(editor, runtime),
            legacy_profiling_status(editor, runtime)
        );
    }
}

#[test]
fn optimization_batch_20260829af_editor251_profiling_status_uses_one_buffer() {
    let source = include_str!("../status.rs");
    let helper = source
        .split("fn single_buffer_profiling_status")
        .nth(1)
        .expect("single-buffer status builder")
        .split("#[cfg(feature")
        .next()
        .expect("status builder body");

    assert!(helper.contains("String::with_capacity"));
    assert!(helper.contains("status.push_str(EDITOR_PREFIX)"));
    assert!(helper.contains("status.push_str(runtime_message)"));
    assert!(!helper.contains("vec!["));
    assert!(!helper.contains("format!("));
    assert!(!helper.contains(".join("));
    assert!(source.contains("single_buffer_profiling_status(&editor_response.message"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829af_editor251_single_buffer_profiling_status_bench() {
    let editor = "timeline capture completed with editor counters and UI spans";
    let runtime = "timeline capture completed with runtime counters and render spans";
    assert_eq!(
        single_buffer_profiling_status(editor, runtime),
        legacy_profiling_status(editor, runtime)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, editor, runtime));
            optimized_samples.push(measure(true, editor, runtime));
        } else {
            optimized_samples.push(measure(true, editor, runtime));
            legacy_samples.push(measure(false, editor, runtime));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR251_SINGLE_BUFFER_PROFILING_STATUS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
messages_per_sample={MESSAGES_PER_SAMPLE} editor_bytes={} runtime_bytes={} \
legacy_result_buffers_per_message=3 optimized_result_buffers_per_message=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        editor.len(),
        runtime.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_profiling_status(editor_message: &str, runtime_message: &str) -> String {
    let parts = vec![
        format!("Editor profiling: {editor_message}"),
        format!("Runtime profiling: {runtime_message}"),
    ];
    parts.join("; ")
}

fn measure(optimized: bool, editor: &str, runtime: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..MESSAGES_PER_SAMPLE {
        let message = if optimized {
            single_buffer_profiling_status(black_box(editor), black_box(runtime))
        } else {
            legacy_profiling_status(black_box(editor), black_box(runtime))
        };
        checksum = checksum.wrapping_add(black_box(message).len());
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
