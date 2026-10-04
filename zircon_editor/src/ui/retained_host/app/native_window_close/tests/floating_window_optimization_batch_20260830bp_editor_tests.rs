use std::hint::black_box;
use std::time::Instant;

use crate::ui::workbench::layout::{DocumentNode, TabStackLayout};
use crate::ui::workbench::view::ViewInstanceId;

const PERF_MARKER: &str = "EDITOR314_FLOATING_WINDOW_INSTANCE_CAPACITY_BENCH_V1";

#[test]
fn floating_window_instance_collection_preserves_depth_first_order() {
    let node: DocumentNode = serde_json::from_value(serde_json::json!({
        "SplitNode": {
            "axis": "Horizontal",
            "ratio": 0.5,
            "first": { "Tabs": { "tabs": ["first-a", "first-b"], "active_tab": null } },
            "second": { "Tabs": { "tabs": ["second-a"], "active_tab": null } }
        }
    }))
    .expect("serialized split layout should decode");

    let mut instances = Vec::with_capacity(node.instance_count());
    node.append_instance_ids(&mut instances);
    assert_eq!(
        instances,
        vec![
            serde_json::from_value(serde_json::json!("first-a")).unwrap(),
            serde_json::from_value(serde_json::json!("first-b")).unwrap(),
            serde_json::from_value(serde_json::json!("second-a")).unwrap(),
        ]
    );
    assert_eq!(instances.capacity(), 3);
}

#[test]
fn floating_window_instance_collection_uses_recursive_capacity_count() {
    let source = include_str!("../../../../workbench/layout/document_node.rs");
    assert!(source.contains("pub(crate) fn instance_count"));
    assert!(source.contains("pub(crate) fn append_instance_ids"));
    assert!(source.contains("out.extend(stack.tabs.iter().cloned())"));
}

#[test]
#[ignore = "release performance evidence"]
fn floating_window_instance_collection_capacity_p95() {
    const TABS: usize = 4_096;
    const SAMPLES: usize = 17;
    let node = black_box(DocumentNode::tabs(TabStackLayout {
        tabs: (0..TABS)
            .map(|index| ViewInstanceId::new(format!("view-{index}")))
            .collect(),
        active_tab: None,
    }));
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..256 {
                let mut instances = if pass == 0 {
                    Vec::new()
                } else {
                    Vec::with_capacity(node.instance_count())
                };
                node.append_instance_ids(&mut instances);
                checksum = checksum.wrapping_add(instances.len());
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} tabs={TABS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
