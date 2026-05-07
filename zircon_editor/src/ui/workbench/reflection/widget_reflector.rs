use std::collections::BTreeSet;

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
        let mut visited = BTreeSet::new();
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
        visited: &mut BTreeSet<UiNodeId>,
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
