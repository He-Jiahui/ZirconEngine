use std::collections::HashSet;

use zircon_runtime_interface::ui::event_ui::{
    UiNodeId, UiNodePath, UiReflectedProperty, UiReflectorHitContext, UiReflectorNode,
    UiReflectorSnapshot, UiWidgetLifecycleState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchWidgetReflectorModel {
    snapshot: UiReflectorSnapshot,
    selected_node: Option<UiNodeId>,
}

impl WorkbenchWidgetReflectorModel {
    pub fn new(snapshot: UiReflectorSnapshot) -> Self {
        Self {
            snapshot,
            selected_node: None,
        }
    }

    pub fn snapshot(&self) -> &UiReflectorSnapshot {
        &self.snapshot
    }

    pub fn export_snapshot(&self) -> &UiReflectorSnapshot {
        &self.snapshot
    }

    pub fn into_snapshot(self) -> UiReflectorSnapshot {
        self.snapshot
    }

    pub fn hit_context(&self) -> Option<&UiReflectorHitContext> {
        self.snapshot.hit_context.as_ref()
    }

    pub fn selected_node_id(&self) -> Option<UiNodeId> {
        self.selected_node
    }

    pub fn set_selected_node(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<(), WorkbenchWidgetReflectorError> {
        if self.snapshot.nodes.contains_key(&node_id) {
            self.selected_node = Some(node_id);
            Ok(())
        } else {
            Err(WorkbenchWidgetReflectorError::MissingNode(node_id))
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_node = None;
    }

    pub fn selected(&self) -> Option<WorkbenchWidgetReflectorSelection<'_>> {
        let node = self
            .selected_node
            .and_then(|node| self.snapshot.node(node))?;
        Some(WorkbenchWidgetReflectorSelection {
            node,
            properties: node.properties.values().collect(),
        })
    }

    pub fn rows(&self) -> Vec<WorkbenchWidgetReflectorRow> {
        let mut rows = Vec::new();
        let mut visited = HashSet::new();
        for root in &self.snapshot.roots {
            self.push_rows(*root, 0, &mut visited, &mut rows);
        }
        for node_id in self.snapshot.nodes.keys() {
            if !visited.contains(node_id) {
                self.push_rows(*node_id, 0, &mut visited, &mut rows);
            }
        }
        rows
    }

    fn push_rows(
        &self,
        node_id: UiNodeId,
        depth: usize,
        visited: &mut HashSet<UiNodeId>,
        rows: &mut Vec<WorkbenchWidgetReflectorRow>,
    ) {
        if !visited.insert(node_id) {
            return;
        }
        let Some(node) = self.snapshot.node(node_id) else {
            return;
        };
        rows.push(WorkbenchWidgetReflectorRow::from_node(node, depth));
        for child in &node.children {
            self.push_rows(*child, depth + 1, visited, rows);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkbenchWidgetReflectorError {
    MissingNode(UiNodeId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchWidgetReflectorRow {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub depth: usize,
    pub class_name: String,
    pub display_name: String,
    pub lifecycle: UiWidgetLifecycleState,
    pub visible: bool,
    pub enabled: bool,
    pub dirty: bool,
    pub focused: bool,
    pub hovered: bool,
    pub captured: bool,
}

impl WorkbenchWidgetReflectorRow {
    fn from_node(node: &UiReflectorNode, depth: usize) -> Self {
        Self {
            node_id: node.node_id,
            node_path: node.node_path.clone(),
            parent: node.parent,
            depth,
            class_name: node.class_name.clone(),
            display_name: node.display_name.clone(),
            lifecycle: node.lifecycle,
            visible: node.state_flags.visible,
            enabled: node.state_flags.enabled,
            dirty: node.dirty.any() || node.state_flags.dirty,
            focused: node.focused,
            hovered: node.hovered,
            captured: node.captured,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchWidgetReflectorSelection<'a> {
    pub node: &'a UiReflectorNode,
    pub properties: Vec<&'a UiReflectedProperty>,
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiTreeId};

    use super::*;

    const VISIT_COUNT: usize = 65_536;
    const UNIQUE_NODE_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn node_visits() -> Vec<UiNodeId> {
        (0..VISIT_COUNT)
            .map(|index| UiNodeId::new(((index * 4_099) % UNIQUE_NODE_COUNT) as u64))
            .collect()
    }

    fn ordered_visit_count(visits: &[UiNodeId]) -> usize {
        let mut visited = BTreeSet::new();
        visits
            .iter()
            .filter(|node_id| visited.insert(**node_id))
            .count()
    }

    fn hash_visit_count(visits: &[UiNodeId]) -> usize {
        let mut visited = HashSet::with_capacity(UNIQUE_NODE_COUNT);
        visits
            .iter()
            .filter(|node_id| visited.insert(**node_id))
            .count()
    }

    fn node(id: u64, children: Vec<UiNodeId>) -> UiReflectorNode {
        let mut node = UiReflectorNode::new(
            UiNodeId::new(id),
            UiNodePath::new(format!("root/{id}")),
            "Panel",
            format!("Node {id}"),
        );
        node.children = children;
        node
    }

    #[test]
    fn optimization_batch_20260826w_editor25_hash_visited_preserves_tree_and_orphan_order() {
        let snapshot = UiReflectorSnapshot::new(
            UiTreeId::new("editor.reflector.optimization"),
            vec![UiNodeId::new(30), UiNodeId::new(30)],
            vec![
                node(30, vec![UiNodeId::new(10)]),
                node(10, vec![UiNodeId::new(20)]),
                node(20, vec![UiNodeId::new(30)]),
                node(5, Vec::new()),
            ],
        );

        let rows = WorkbenchWidgetReflectorModel::new(snapshot).rows();
        assert_eq!(
            rows.iter().map(|row| row.node_id).collect::<Vec<_>>(),
            vec![
                UiNodeId::new(30),
                UiNodeId::new(10),
                UiNodeId::new(20),
                UiNodeId::new(5),
            ]
        );
        assert_eq!(
            rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
            vec![0, 1, 2, 0]
        );
    }

    #[test]
    fn optimization_batch_20260826w_editor25_widget_reflector_uses_hash_visited_membership() {
        let source = include_str!("widget_reflector.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("let mut visited = HashSet::new();"));
        assert!(production.contains("visited: &mut HashSet<UiNodeId>"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826w_editor25_widget_reflector_hash_visited_performance_evidence() {
        let visits = node_visits();
        assert_eq!(ordered_visit_count(&visits), hash_visit_count(&visits));

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_visit_count(black_box(&visits)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_visit_count(black_box(&visits)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_visit_count(black_box(&visits)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_visit_count(black_box(&visits)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR25_WIDGET_REFLECTOR_HASH_VISITED_BENCH_V1 visits={VISIT_COUNT} \
             unique_nodes={UNIQUE_NODE_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-visited P95 {:?} exceeded 60% of ordered-visited P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
