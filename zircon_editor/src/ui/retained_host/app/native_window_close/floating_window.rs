use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::{
    layout::{DocumentNode, LayoutCommand, MainPageId},
    view::ViewInstanceId,
};

use super::super::{RetainedEditorHost, callback_dispatch};

impl RetainedEditorHost {
    pub(super) fn close_floating_window_without_prompt(
        &mut self,
        window_id: &MainPageId,
        instance_ids: Vec<ViewInstanceId>,
    ) -> CloseRequestResponse {
        for instance_id in instance_ids {
            match callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::CloseView { instance_id },
            ) {
                Ok(effects) => self.apply_dispatch_effects(effects),
                Err(error) => {
                    self.set_status_line(error);
                    return CloseRequestResponse::KeepWindowShown;
                }
            }
        }

        self.recompute_if_dirty();
        let window_still_exists = self
            .runtime
            .current_layout()
            .floating_windows
            .iter()
            .any(|window| &window.window_id == window_id);
        if window_still_exists {
            CloseRequestResponse::KeepWindowShown
        } else {
            CloseRequestResponse::HideWindow
        }
    }

    pub(super) fn floating_window_close_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        let layout = self.runtime.current_layout();
        let window = layout
            .floating_windows
            .iter()
            .find(|window| &window.window_id == window_id)?;
        let mut instances = Vec::with_capacity(document_node_instance_count(&window.workspace));
        collect_document_node_instances(&window.workspace, &mut instances);
        (!instances.is_empty()).then_some(instances)
    }
}

fn document_node_instance_count(node: &DocumentNode) -> usize {
    match node {
        DocumentNode::Tabs(stack) => stack.tabs.len(),
        DocumentNode::SplitNode { first, second, .. } => {
            document_node_instance_count(first).saturating_add(document_node_instance_count(second))
        }
    }
}

fn collect_document_node_instances(node: &DocumentNode, out: &mut Vec<ViewInstanceId>) {
    match node {
        DocumentNode::Tabs(stack) => out.extend(stack.tabs.iter().cloned()),
        DocumentNode::SplitNode { first, second, .. } => {
            collect_document_node_instances(first, out);
            collect_document_node_instances(second, out);
        }
    }
}

#[cfg(test)]
mod optimization_batch_20260830bp_editor_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::ui::workbench::layout::DocumentNode;

    use super::{collect_document_node_instances, document_node_instance_count};

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

        let mut instances = Vec::with_capacity(document_node_instance_count(&node));
        collect_document_node_instances(&node, &mut instances);
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
        let source = include_str!("floating_window.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(production.contains("document_node_instance_count(&window.workspace)"));
        assert!(production.contains("Vec::with_capacity(document_node_instance_count"));
        assert!(!production.contains("let mut instances = Vec::new();"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn floating_window_instance_collection_capacity_p95() {
        const TABS: usize = 4_096;
        const SAMPLES: usize = 17;
        let node = black_box(DocumentNode::Tabs(TabStackLayout {
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
                        Vec::with_capacity(document_node_instance_count(&node))
                    };
                    collect_document_node_instances(&node, &mut instances);
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
}
