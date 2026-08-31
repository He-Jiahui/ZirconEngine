use std::collections::HashMap;

use crate::ui::retained_host::{primitives::ModelRc, TemplatePaneNodeData};

#[derive(Debug, PartialEq, Eq)]
struct WorkbenchProjectionIdentity {
    document_id: String,
    row_by_control_id: HashMap<String, usize>,
}

pub(super) struct PreviousWorkbenchNodeIndex<'a> {
    nodes: &'a ModelRc<TemplatePaneNodeData>,
    row_by_control_id: &'a HashMap<String, usize>,
}

impl<'a> PreviousWorkbenchNodeIndex<'a> {
    pub(super) fn for_projection(
        nodes: &'a ModelRc<TemplatePaneNodeData>,
        document_id: &str,
    ) -> Option<Self> {
        let identity = nodes.metadata::<WorkbenchProjectionIdentity>()?;
        (identity.document_id.as_str() == document_id).then_some(Self {
            nodes,
            row_by_control_id: &identity.row_by_control_id,
        })
    }

    pub(super) fn get(&self, control_id: &str) -> Option<&'a TemplatePaneNodeData> {
        self.nodes.get(self.row(control_id)?)
    }

    pub(super) fn row(&self, control_id: &str) -> Option<usize> {
        self.row_by_control_id.get(control_id).copied()
    }
}

pub(super) fn model_with_projection_identity(
    nodes: Vec<TemplatePaneNodeData>,
    document_id: String,
) -> ModelRc<TemplatePaneNodeData> {
    let row_by_control_id = projection_rows_by_control_id(&nodes);
    ModelRc::with_metadata(
        nodes,
        WorkbenchProjectionIdentity {
            document_id,
            row_by_control_id,
        },
    )
}

fn projection_rows_by_control_id(nodes: &[TemplatePaneNodeData]) -> HashMap<String, usize> {
    let mut rows = HashMap::with_capacity(nodes.len());
    for (row, node) in nodes.iter().enumerate() {
        if !node.control_id.is_empty() {
            rows.insert(node.control_id.to_string(), row);
        }
    }
    rows
}

#[cfg(test)]
mod optimization_tests {
    use super::{projection_rows_by_control_id, TemplatePaneNodeData};

    #[test]
    fn optimization_batch_20260830cv_projection_row_capacity_preserves_last_duplicate_row() {
        let nodes = test_nodes(["", "viewport", "inspector", "viewport"]);
        let rows = projection_rows_by_control_id(&nodes);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows.get("viewport"), Some(&3));
        assert_eq!(rows.get("inspector"), Some(&2));
        assert!(!rows.contains_key(""));
    }

    #[test]
    fn optimization_batch_20260830cv_projection_row_capacity_source_contract() {
        let source = include_str!("previous_node_index.rs");
        let projection = source
            .split("fn projection_rows_by_control_id")
            .nth(1)
            .expect("projection row index implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("bounded projection row index implementation");

        assert!(projection.contains("HashMap::with_capacity(nodes.len())"));
        assert!(projection.contains("for (row, node) in nodes.iter().enumerate()"));
        assert!(!projection.contains(".collect()"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260830cv_editor_projection_row_capacity_p95() {
        fn legacy_rows(nodes: &[TemplatePaneNodeData]) -> std::collections::HashMap<String, usize> {
            nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| !node.control_id.is_empty())
                .map(|(row, node)| (node.control_id.to_string(), row))
                .collect()
        }

        fn measure(
            nodes: &[TemplatePaneNodeData],
            project: impl Fn(&[TemplatePaneNodeData]) -> std::collections::HashMap<String, usize>,
        ) -> u128 {
            let started = std::time::Instant::now();
            for _ in 0..8 {
                std::hint::black_box(project(std::hint::black_box(nodes)));
            }
            started.elapsed().as_nanos()
        }

        let nodes = test_nodes((0..32_768).map(|index| {
            if index % 8 == 0 {
                String::new()
            } else {
                format!("workbench.control.{index:05}")
            }
        }));
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for sample_index in 0..17 {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure(&nodes, legacy_rows));
                optimized_samples.push(measure(&nodes, projection_rows_by_control_id));
            } else {
                optimized_samples.push(measure(&nodes, projection_rows_by_control_id));
                legacy_samples.push(measure(&nodes, legacy_rows));
            }
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "EDITOR338_PROJECTION_ROW_CAPACITY_BENCH_V1 nodes={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            nodes.len(),
            legacy_p95,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "capacity-sized projection index P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }

    fn test_nodes<I, S>(ids: I) -> Vec<TemplatePaneNodeData>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        ids.into_iter()
            .map(|control_id| {
                let control_id: String = control_id.into();
                TemplatePaneNodeData {
                    control_id: control_id.into(),
                    ..TemplatePaneNodeData::default()
                }
            })
            .collect()
    }
}
