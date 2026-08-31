use std::hint::black_box;
use std::time::Instant;

use serde_json::Value;

use super::{activity_from_tab, collect_workspace_activities, workspace_activity_count};
use crate::ui::workbench::layout::{MainPageId, SplitAxis};
use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, ViewContentKind, ViewTabSnapshot};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId, ViewKind};
use crate::ui::EditorActivityHost;

const SAMPLE_PAIRS: usize = 21;
const COLLECTIONS_PER_SAMPLE: usize = 16;
const TABS_PER_WORKSPACE: usize = 256;

#[test]
fn optimization_batch_20260828io_editor233_single_pass_preserves_activity_order() {
    let workspace = right_skewed_workspace(0, 16);

    let activities = collect_workspace_activities(&workspace, activity_host());

    assert_eq!(workspace_activity_count(&workspace), 16);
    assert_eq!(activities.len(), 16);
    assert_eq!(activities[0].instance_id, "editor233.tab-0");
    assert_eq!(activities[15].instance_id, "editor233.tab-15");
    for (index, activity) in activities.iter().enumerate() {
        assert_eq!(activity.instance_id, format!("editor233.tab-{index}"));
    }
}

#[test]
fn optimization_batch_20260828io_editor233_workspace_activities_use_one_output_vec() {
    let source = include_str!("../activity_collection.rs");

    assert!(source.contains("Vec::with_capacity(workspace_activity_count(workspace))"));
    assert!(source.contains("collect_workspace_activities_into(first, host, activities)"));
    assert!(source.contains("collect_workspace_activities_into(second, host, activities)"));
    assert!(!source.contains("activities.extend(collect_workspace_activities(second, host))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828io_editor233_single_pass_workspace_activities_bench() {
    let workspace = right_skewed_workspace(0, TABS_PER_WORKSPACE);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&workspace, false));
            optimized_samples.push(measure(&workspace, true));
        } else {
            optimized_samples.push(measure(&workspace, true));
            legacy_samples.push(measure(&workspace, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR233_SINGLE_PASS_WORKSPACE_ACTIVITIES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
collections_per_sample={COLLECTIONS_PER_SAMPLE} tabs_per_workspace={TABS_PER_WORKSPACE} \
legacy_intermediate_vectors_per_collection={} optimized_intermediate_vectors_per_collection=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        TABS_PER_WORKSPACE - 1,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_collect_workspace_activities(
    workspace: &DocumentWorkspaceSnapshot,
    host: EditorActivityHost,
) -> Vec<crate::ui::EditorActivityReflection> {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            let mut activities = legacy_collect_workspace_activities(first, host.clone());
            activities.extend(legacy_collect_workspace_activities(second, host));
            activities
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs
            .iter()
            .map(|tab| activity_from_tab(tab, host.clone()))
            .collect(),
    }
}

fn right_skewed_workspace(start: usize, count: usize) -> DocumentWorkspaceSnapshot {
    assert!(count > 0);
    if count == 1 {
        return tab_workspace(start);
    }
    DocumentWorkspaceSnapshot::Split {
        axis: SplitAxis::Horizontal,
        ratio: 0.5,
        first: Box::new(tab_workspace(start)),
        second: Box::new(right_skewed_workspace(start + 1, count - 1)),
    }
}

fn tab_workspace(index: usize) -> DocumentWorkspaceSnapshot {
    DocumentWorkspaceSnapshot::Tabs {
        tabs: vec![ViewTabSnapshot {
            instance_id: ViewInstanceId::new(format!("editor233.tab-{index}")),
            descriptor_id: ViewDescriptorId::new("editor233.placeholder"),
            title: format!("Placeholder {index}"),
            icon_key: "placeholder".to_string(),
            kind: ViewKind::ActivityView,
            host: ViewHost::Document(MainPageId::workbench(), Vec::new()),
            serializable_payload: Value::Null,
            dirty: false,
            content_kind: ViewContentKind::Placeholder,
            pane_template: None,
            activity_window_template: None,
            placeholder: true,
        }],
        active_tab: None,
    }
}

fn activity_host() -> EditorActivityHost {
    EditorActivityHost::DocumentPage("editor233-workbench-host".repeat(4))
}

fn measure(workspace: &DocumentWorkspaceSnapshot, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..COLLECTIONS_PER_SAMPLE {
        let activities = if optimized {
            collect_workspace_activities(black_box(workspace), activity_host())
        } else {
            legacy_collect_workspace_activities(black_box(workspace), activity_host())
        };
        checksum ^= black_box(activities.len());
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
