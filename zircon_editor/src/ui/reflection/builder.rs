use std::collections::BTreeMap;

use zircon_runtime_interface::ui::event_ui::{
    UiActionDescriptor, UiNodeDescriptor, UiNodeId, UiNodePath, UiPropertyDescriptor,
    UiReflectionSnapshot, UiStateFlags, UiTreeId,
};

pub(super) struct SnapshotBuilder {
    tree_id: UiTreeId,
    next_id: u64,
    nodes: BTreeMap<UiNodeId, UiNodeDescriptor>,
}

impl SnapshotBuilder {
    pub(super) fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            next_id: 0,
            nodes: BTreeMap::new(),
        }
    }

    pub(super) fn push_node(
        &mut self,
        path: impl Into<String>,
        class_name: impl Into<String>,
        display_name: impl Into<String>,
        state_flags: UiStateFlags,
        properties: Vec<UiPropertyDescriptor>,
        actions: Vec<UiActionDescriptor>,
    ) -> UiNodeId {
        self.next_id += 1;
        let node_id = UiNodeId::new(self.next_id);
        let mut node =
            UiNodeDescriptor::new(node_id, UiNodePath::new(path), class_name, display_name)
                .with_state_flags(state_flags);
        for property in properties {
            node = node.with_property(property);
        }
        for action in actions {
            node = node.with_action(action);
        }
        self.nodes.insert(node_id, node);
        node_id
    }

    pub(super) fn add_child(&mut self, parent: UiNodeId, child: UiNodeId) {
        if let Some(node) = self.nodes.get_mut(&parent) {
            node.children.push(child);
        }
    }

    pub(super) fn finish(self, root: UiNodeId) -> UiReflectionSnapshot {
        UiReflectionSnapshot {
            tree_id: self.tree_id,
            roots: vec![root],
            nodes: self.nodes,
        }
    }
}
