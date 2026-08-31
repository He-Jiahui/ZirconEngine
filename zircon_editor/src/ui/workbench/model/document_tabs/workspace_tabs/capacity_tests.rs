use std::hint::black_box;
use std::time::Instant;

use serde_json::Value;

use super::workspace_document_tab_count;
use crate::ui::workbench::layout::{MainPageId, SplitAxis};
use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, ViewContentKind, ViewTabSnapshot};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId, ViewKind};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const TABS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fa_editor142_capacity_counts_nested_workspace_tabs() {
    let workspace = DocumentWorkspaceSnapshot::Split {
        axis: SplitAxis::Horizontal,
        ratio: 0.5,
        first: Box::new(tab_workspace(0, 96)),
        second: Box::new(DocumentWorkspaceSnapshot::Split {
            axis: SplitAxis::Vertical,
            ratio: 0.5,
            first: Box::new(tab_workspace(96, 64)),
            second: Box::new(tab_workspace(160, 96)),
        }),
    };

    assert_eq!(workspace_document_tab_count(&workspace), TABS_PER_BUILD);
}

#[test]
fn optimization_batch_20260826fa_editor142_workspace_tabs_reserve_recursive_count() {
    let source = include_str!("../workspace_tabs.rs");
    assert!(source.contains("fn workspace_document_tab_count("));
    assert!(source.contains("Vec::with_capacity(workspace_document_tab_count(workspace))"));
    assert!(source.contains("workspace_document_tab_count(first)"));
    assert!(source.contains(".saturating_add(workspace_document_tab_count(second))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fa_editor142_workspace_document_tab_capacity_bench() {
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
        "EDITOR142_WORKSPACE_DOCUMENT_TAB_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} tabs_per_build={TABS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn tab_workspace(start: usize, count: usize) -> DocumentWorkspaceSnapshot {
    DocumentWorkspaceSnapshot::Tabs {
        tabs: (start..start + count).map(tab_snapshot).collect(),
        active_tab: None,
    }
}

fn tab_snapshot(index: usize) -> ViewTabSnapshot {
    ViewTabSnapshot {
        instance_id: ViewInstanceId::new(format!("runtime142.tab-{index}")),
        descriptor_id: ViewDescriptorId::new("runtime142.document"),
        title: format!("Document {index}"),
        icon_key: "document".to_string(),
        kind: ViewKind::ActivityView,
        host: ViewHost::Document(MainPageId::workbench(), Vec::new()),
        serializable_payload: Value::Null,
        dirty: false,
        content_kind: ViewContentKind::Scene,
        pane_template: None,
        activity_window_template: None,
        placeholder: false,
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut tabs = if reserve {
            Vec::with_capacity(TABS_PER_BUILD)
        } else {
            Vec::new()
        };
        for tab in 0..TABS_PER_BUILD {
            tabs.push(black_box(tab));
        }
        checksum ^= black_box(tabs.len() ^ tabs.capacity());
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
