use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::EditorCommandId;

use super::EditorOperationPath;

const CHECKS_PER_SAMPLE: usize = 8192;
const PATH_BYTES: usize = 4096;
const SAMPLE_PAIRS: usize = 31;

fn legacy_is_valid(value: &str) -> bool {
    let mut segment_count = 0;
    let valid = value.split('.').all(|segment| {
        segment_count += 1;
        !segment.is_empty()
            && segment
                .chars()
                .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '_')
    });
    valid && segment_count >= 3
}

fn optimized_is_valid(value: &str) -> bool {
    EditorCommandId::is_valid(value)
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        valid += usize::from(if optimized {
            optimized_is_valid(black_box(value))
        } else {
            legacy_is_valid(black_box(value))
        });
    }
    black_box(valid);
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
fn optimization_batch_20260829bf_editor278_single_scan_paths_preserve_results() {
    for value in [
        "editor.asset.open",
        "editor.asset_2.open",
        "",
        "editor",
        "editor.asset",
        ".editor.asset",
        "editor..asset",
        "editor.asset.",
        "Editor.asset.open",
        "editor.asset-open",
        "editor.asset.\u{4f8b}",
    ] {
        assert_eq!(
            optimized_is_valid(value),
            legacy_is_valid(value),
            "{value:?}"
        );
        assert_eq!(
            EditorOperationPath::parse(value).is_ok(),
            legacy_is_valid(value)
        );
    }
}

#[test]
fn optimization_batch_20260829bf_editor278_operation_path_uses_one_byte_scan() {
    let host_source = include_str!("../editor_operation.rs");
    let shared_source =
        include_str!("../../../../zircon_runtime_interface/src/editor_command_id.rs");

    assert!(host_source.contains("EditorCommandId::parse(value)"));
    assert!(shared_source.contains("for byte in value.bytes()"));
    assert!(EditorCommandId::is_valid("editor.asset.open"));
    assert!(!EditorCommandId::is_valid("editor.asset-open"));
    assert!(!shared_source.contains("value.split('.')"));
    assert!(!shared_source.contains("segment.chars()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bf_editor278_single_scan_operation_path_bench() {
    let suffix = ".asset.open";
    let value = format!("{}{}", "a".repeat(PATH_BYTES - suffix.len()), suffix);
    assert_eq!(value.len(), PATH_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&value, false));
            optimized_samples.push(measure(&value, true));
        } else {
            optimized_samples.push(measure(&value, true));
            legacy_samples.push(measure(&value, false));
        }
    }

    let baseline_p50_ns = percentile(&legacy_samples, 50);
    let candidate_p50_ns = percentile(&optimized_samples, 50);
    let baseline_p95_ns = percentile(&legacy_samples, 95);
    let candidate_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR278_SINGLE_SCAN_OPERATION_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} path_bytes={PATH_BYTES} \
legacy_path_scans=2 optimized_path_scans=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
