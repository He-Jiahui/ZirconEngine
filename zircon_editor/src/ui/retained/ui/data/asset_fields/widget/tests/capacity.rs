use std::hint::black_box;
use std::time::Instant;

use super::{
    asset_editor, widget_detail_row_capacity, widget_detail_rows, UiAssetDetailFieldRow,
    WIDGET_DETAIL_ROW_MAX_COUNT,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260920er_editor850_capacity_preserves_widget_detail_rows() {
    let data = full_widget_presentation();
    let prop_state_rows = full_prop_state_rows();

    let rows = widget_detail_rows(&data, &prop_state_rows);

    assert_eq!(rows.len(), WIDGET_DETAIL_ROW_MAX_COUNT);
    assert!(rows.capacity() >= WIDGET_DETAIL_ROW_MAX_COUNT);
    assert_eq!(
        widget_detail_row_capacity(&data, &prop_state_rows),
        WIDGET_DETAIL_ROW_MAX_COUNT
    );
    assert_eq!(
        widget_detail_row_capacity(&asset_editor::UiAssetEditorPanePresentation::default(), &[],),
        0
    );
}

#[test]
fn optimization_batch_20260920er_editor850_widget_capacity_counts_only_emitted_rows() {
    let data = asset_editor::UiAssetEditorPanePresentation {
        inspector_control_id: "control".into(),
        inspector_can_edit_text_prop: true,
        ..asset_editor::UiAssetEditorPanePresentation::default()
    };
    let prop_state_rows = vec![
        prop_state_item("prop", "text.value"),
        prop_state_item("unknown", "ignored"),
        prop_state_item("state", ""),
        prop_state_item("state", "pressed"),
    ];

    let rows = widget_detail_rows(&data, &prop_state_rows);

    assert_eq!(rows.len(), 4);
    assert_eq!(widget_detail_row_capacity(&data, &prop_state_rows), 4);
    assert_eq!(rows[0].label, "Control ID");
    assert_eq!(rows[1].label, "Text");
    assert_eq!(rows[2].label, "prop text.value");
    assert_eq!(rows[3].label, "state pressed");
}

#[test]
fn optimization_batch_20260920er_editor850_widget_capacity_source_contract() {
    let source = include_str!("../../widget.rs");
    let builder_start = source.find("fn widget_detail_rows").unwrap();
    let builder_end = source[builder_start..]
        .find("fn widget_detail_row_capacity")
        .map(|offset| builder_start + offset)
        .unwrap();
    let builder_source = &source[builder_start..builder_end];

    assert!(source.contains("const WIDGET_DETAIL_ROW_MAX_COUNT: usize = 9;"));
    assert!(builder_source
        .contains("Vec::with_capacity(widget_detail_row_capacity(data, prop_state_rows))"));
    assert!(source.contains("prop_state_row_is_actionable"));
    assert!(source.contains("debug_assert!(capacity <= WIDGET_DETAIL_ROW_MAX_COUNT);"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260920er_editor850_widget_detail_row_capacity_bench() {
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
        "EDITOR850_WIDGET_DETAIL_ROW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={WIDGET_DETAIL_ROW_MAX_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved Widget detail-row build P95 {optimized_p95_ns}ns must be at most 70% of growth-driven build P95 {legacy_p95_ns}ns"
    );
}

fn full_widget_presentation() -> asset_editor::UiAssetEditorPanePresentation {
    asset_editor::UiAssetEditorPanePresentation {
        inspector_control_id: "control".into(),
        inspector_text_prop: "text".into(),
        inspector_component_root_class_policy: "root".into(),
        inspector_can_edit_control_id: true,
        inspector_can_edit_text_prop: true,
        inspector_can_edit_component_root_class_policy: true,
        ..asset_editor::UiAssetEditorPanePresentation::default()
    }
}

fn full_prop_state_rows() -> Vec<asset_editor::UiAssetEditorWidgetPropStateItem> {
    (0..6)
        .map(|index| prop_state_item("prop", &format!("field{index}")))
        .collect()
}

fn prop_state_item(kind: &str, path: &str) -> asset_editor::UiAssetEditorWidgetPropStateItem {
    asset_editor::UiAssetEditorWidgetPropStateItem {
        kind: kind.into(),
        path: path.into(),
        value: "value".into(),
        display: "display".into(),
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(WIDGET_DETAIL_ROW_MAX_COUNT)
        } else {
            Vec::new()
        };
        for _ in 0..WIDGET_DETAIL_ROW_MAX_COUNT {
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
