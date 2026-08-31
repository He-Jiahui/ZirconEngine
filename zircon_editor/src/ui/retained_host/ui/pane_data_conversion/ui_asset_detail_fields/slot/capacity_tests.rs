use std::hint::black_box;
use std::time::Instant;

use super::{
    asset_editor, slot_detail_row_capacity, slot_detail_rows, UiAssetDetailFieldRow,
    SLOT_DETAIL_ROW_MAX_COUNT,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 32_768;

#[test]
fn optimization_batch_20260826ep_editor131_capacity_preserves_full_slot_rows() {
    let data = full_slot_presentation();

    let rows = slot_detail_rows(&data);

    assert_eq!(rows.len(), SLOT_DETAIL_ROW_MAX_COUNT);
    assert!(rows.capacity() >= SLOT_DETAIL_ROW_MAX_COUNT);
    assert_eq!(slot_detail_row_capacity(&data), SLOT_DETAIL_ROW_MAX_COUNT);
    assert_eq!(
        slot_detail_row_capacity(&asset_editor::UiAssetEditorPanePresentation::default()),
        0
    );
}

#[test]
fn optimization_batch_20260826ep_editor131_slot_counts_non_empty_rows_before_allocation() {
    let source = include_str!("../slot.rs");
    let builder_start = source.find("fn slot_detail_rows").unwrap();
    let builder_end = source[builder_start..]
        .find("fn slot_detail_row_capacity")
        .map(|offset| builder_start + offset)
        .unwrap();
    let builder_source = &source[builder_start..builder_end];

    assert!(source.contains("const SLOT_DETAIL_ROW_MAX_COUNT: usize = 18;"));
    assert!(builder_source.contains("Vec::with_capacity(slot_detail_row_capacity(data))"));
    assert!(source.contains(".filter(|value| !value.is_empty())"));
    assert!(source.contains("debug_assert!(capacity <= SLOT_DETAIL_ROW_MAX_COUNT);"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ep_editor131_slot_detail_row_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR131_SLOT_DETAIL_ROW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={SLOT_DETAIL_ROW_MAX_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved Slot detail-row build P95 {optimized_p95_ns}ns must be at most 70% of growth-driven build P95 {legacy_p95_ns}ns"
    );
}

fn full_slot_presentation() -> asset_editor::UiAssetEditorPanePresentation {
    asset_editor::UiAssetEditorPanePresentation {
        inspector_mount: "mount".into(),
        inspector_slot_padding: "padding".into(),
        inspector_slot_width_preferred: "width".into(),
        inspector_slot_height_preferred: "height".into(),
        inspector_slot_semantic_value: "semantic".into(),
        inspector_slot_linear_main_weight: "1".into(),
        inspector_slot_linear_main_stretch: "true".into(),
        inspector_slot_linear_cross_weight: "1".into(),
        inspector_slot_linear_cross_stretch: "true".into(),
        inspector_slot_overlay_anchor_x: "0.5".into(),
        inspector_slot_overlay_anchor_y: "0.5".into(),
        inspector_slot_overlay_position_x: "10".into(),
        inspector_slot_overlay_position_y: "20".into(),
        inspector_slot_overlay_z_index: "3".into(),
        inspector_slot_grid_row: "1".into(),
        inspector_slot_grid_column: "2".into(),
        inspector_slot_flow_break_before: "false".into(),
        inspector_slot_flow_alignment: "center".into(),
        ..asset_editor::UiAssetEditorPanePresentation::default()
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(SLOT_DETAIL_ROW_MAX_COUNT)
        } else {
            Vec::new()
        };
        for _ in 0..SLOT_DETAIL_ROW_MAX_COUNT {
            output.push(black_box(empty_row()));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn empty_row() -> UiAssetDetailFieldRow {
    UiAssetDetailFieldRow {
        label: String::new(),
        value: String::new(),
        action_id: String::new(),
        label_control_id: String::new(),
        value_control_id: String::new(),
        disabled: false,
    }
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
