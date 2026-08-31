use std::hint::black_box;
use std::time::Instant;

use super::{binding_detail_rows, UiAssetDetailFieldRow};
use crate::ui::asset_editor::UiAssetEditorPanePresentation;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 104_858;
const ROWS_PER_BUILD: usize = 5;

#[test]
fn optimization_batch_20260826ez_editor141_capacity_preserves_binding_detail_rows() {
    let data = UiAssetEditorPanePresentation {
        inspector_binding_id: "save-binding".to_string(),
        inspector_binding_event: "click".to_string(),
        inspector_binding_route: "command".to_string(),
        inspector_binding_route_target: "document.save".to_string(),
        inspector_binding_action_target: "active-document".to_string(),
        inspector_can_edit_binding: true,
        ..UiAssetEditorPanePresentation::default()
    };

    let rows = binding_detail_rows(&data);

    assert_eq!(rows.len(), ROWS_PER_BUILD);
    assert!(rows.capacity() >= ROWS_PER_BUILD);
    assert_eq!(rows[0].label, "Binding ID");
    assert_eq!(rows[0].value, "save-binding");
    assert_eq!(rows[0].action_id, "binding.id.set");
    assert_eq!(rows[ROWS_PER_BUILD - 1].label, "Action target");
    assert_eq!(rows[ROWS_PER_BUILD - 1].value, "active-document");
}

#[test]
fn optimization_batch_20260826ez_editor141_binding_rows_reserve_fixed_upper_bound() {
    let source = include_str!("../binding.rs");
    assert!(source.contains("const BINDING_DETAIL_ROW_CAPACITY: usize = 5;"));
    assert!(source.contains("Vec::with_capacity(BINDING_DETAIL_ROW_CAPACITY)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ez_editor141_binding_detail_row_capacity_bench() {
    let row = UiAssetDetailFieldRow {
        label: String::new(),
        value: String::new(),
        action_id: String::new(),
        label_control_id: String::new(),
        value_control_id: String::new(),
        disabled: false,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&row, false));
            optimized_samples.push(measure(&row, true));
        } else {
            optimized_samples.push(measure(&row, true));
            legacy_samples.push(measure(&row, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR141_BINDING_DETAIL_ROW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={ROWS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(row: &UiAssetDetailFieldRow, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut rows = if reserve {
            Vec::with_capacity(ROWS_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..ROWS_PER_BUILD {
            rows.push(black_box(row));
        }
        checksum ^= black_box(rows.len() ^ rows.capacity());
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
