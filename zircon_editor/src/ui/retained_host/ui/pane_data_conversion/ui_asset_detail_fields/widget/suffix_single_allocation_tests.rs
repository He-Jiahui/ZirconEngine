use std::hint::black_box;
use std::time::Instant;

use super::{asset_editor, sanitized_prop_state_control_suffix};

const SAMPLE_PAIRS: usize = 21;
const TRANSFORMS_PER_SAMPLE: usize = 8_192;

#[test]
fn optimization_batch_20260826eg_editor122_suffix_preserves_ascii_and_unicode_mapping() {
    let ascii = prop_state_item("prop", "transform.position-x[0]");
    assert_eq!(
        sanitized_prop_state_control_suffix(&ascii, 7),
        "proptransform_position_x_0_"
    );

    let unicode = prop_state_item("state", "a\u{4e2d}b");
    assert_eq!(sanitized_prop_state_control_suffix(&unicode, 7), "statea_b");

    let empty = prop_state_item("", "");
    assert_eq!(sanitized_prop_state_control_suffix(&empty, 7), "7");
}

#[test]
fn optimization_batch_20260826eg_editor122_suffix_uses_single_output_allocation() {
    let source = include_str!("../widget.rs");
    let function_start = source
        .find("fn sanitized_prop_state_control_suffix")
        .unwrap();
    let function_end = source[function_start..]
        .find("#[cfg(test)]")
        .map(|offset| function_start + offset)
        .unwrap();
    let function_source = &source[function_start..function_end];
    assert!(!function_source.contains("format!(\"{}{}\", row.kind, row.path)"));
    assert!(function_source.contains("String::with_capacity"));
    assert_eq!(
        function_source
            .matches("append_sanitized_control_suffix")
            .count(),
        3
    );
    assert!(function_source.contains("value.is_ascii()"));
    assert!(function_source.contains("value.bytes()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eg_editor122_prop_state_suffix_single_allocation_bench() {
    let row = prop_state_item("prop", &"transform.position_x[component]/".repeat(8));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&row));
            optimized_samples.push(measure_optimized(&row));
        } else {
            optimized_samples.push(measure_optimized(&row));
            legacy_samples.push(measure_legacy(&row));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR122_PROP_STATE_SUFFIX_SINGLE_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
transforms_per_sample={TRANSFORMS_PER_SAMPLE} legacy_allocations_per_transform=2 \
optimized_allocations_per_transform=1 optimized_ascii_byte_path=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-allocation ASCII suffix P95 {optimized_p95_ns}ns must be at most 70% of temporary-format P95 {legacy_p95_ns}ns"
    );
}

fn prop_state_item(kind: &str, path: &str) -> asset_editor::UiAssetEditorWidgetPropStateItem {
    asset_editor::UiAssetEditorWidgetPropStateItem {
        kind: kind.to_string(),
        path: path.to_string(),
        value: "value".to_string(),
        display: "display".to_string(),
    }
}

fn legacy_sanitized_prop_state_control_suffix(
    row: &asset_editor::UiAssetEditorWidgetPropStateItem,
    row_index: usize,
) -> String {
    let mut suffix = format!("{}{}", row.kind, row.path)
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if suffix.is_empty() {
        suffix = row_index.to_string();
    }
    suffix
}

fn measure_legacy(row: &asset_editor::UiAssetEditorWidgetPropStateItem) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..TRANSFORMS_PER_SAMPLE {
        checksum ^= black_box(legacy_sanitized_prop_state_control_suffix(
            black_box(row),
            7,
        ))
        .len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(row: &asset_editor::UiAssetEditorWidgetPropStateItem) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..TRANSFORMS_PER_SAMPLE {
        checksum ^= black_box(sanitized_prop_state_control_suffix(black_box(row), 7)).len();
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
