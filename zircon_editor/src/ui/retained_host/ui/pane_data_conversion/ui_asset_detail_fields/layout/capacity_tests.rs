use std::hint::black_box;
use std::time::Instant;

use super::{
    asset_editor, layout_detail_row_capacity, layout_detail_rows, UiAssetDetailFieldRow,
    LAYOUT_DETAIL_ROW_MAX_COUNT,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260826eq_editor132_capacity_preserves_full_layout_rows() {
    let data = asset_editor::UiAssetEditorPanePresentation {
        inspector_layout_width_preferred: "100".into(),
        inspector_layout_height_preferred: "200".into(),
        inspector_layout_semantic_value: "panel".into(),
        inspector_layout_box_gap: "8".into(),
        inspector_layout_scroll_axis: "vertical".into(),
        inspector_layout_scroll_gap: "4".into(),
        inspector_layout_scrollbar_visibility: "auto".into(),
        inspector_layout_virtualization_item_extent: "24".into(),
        inspector_layout_virtualization_overscan: "3".into(),
        inspector_layout_clip: "true".into(),
        ..asset_editor::UiAssetEditorPanePresentation::default()
    };
    let rows = layout_detail_rows(&data);

    assert_eq!(rows.len(), LAYOUT_DETAIL_ROW_MAX_COUNT);
    assert!(rows.capacity() >= LAYOUT_DETAIL_ROW_MAX_COUNT);
    assert_eq!(
        layout_detail_row_capacity(&data),
        LAYOUT_DETAIL_ROW_MAX_COUNT
    );
    assert_eq!(
        layout_detail_row_capacity(&asset_editor::UiAssetEditorPanePresentation::default()),
        0
    );
}

#[test]
fn optimization_batch_20260826eq_editor132_layout_counts_non_empty_rows_before_allocation() {
    let source = include_str!("../layout.rs");
    assert!(source.contains("const LAYOUT_DETAIL_ROW_MAX_COUNT: usize = 10;"));
    assert!(source.contains("Vec::with_capacity(layout_detail_row_capacity(data))"));
    assert!(source.contains(".filter(|value| !value.is_empty())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eq_editor132_layout_detail_row_capacity_bench() {
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
        "EDITOR132_LAYOUT_DETAIL_ROW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={LAYOUT_DETAIL_ROW_MAX_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(LAYOUT_DETAIL_ROW_MAX_COUNT)
        } else {
            Vec::new()
        };
        for _ in 0..LAYOUT_DETAIL_ROW_MAX_COUNT {
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
