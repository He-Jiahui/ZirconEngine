use serde_json::Value;

use super::{diff::compute_diff, UiEventManager};
use zircon_runtime_interface::ui::event_ui::{
    UiInvocationError, UiNodeDescriptor, UiNodePath, UiNotification, UiPropertyDescriptor,
    UiReflectionDiff, UiReflectionSnapshot, UiTreeId,
};

impl UiEventManager {
    pub fn replace_tree(&mut self, snapshot: UiReflectionSnapshot) -> UiReflectionDiff {
        let tree_id = snapshot.tree_id.clone();
        let diff = if let Some(previous) = self.trees.insert(tree_id.clone(), snapshot) {
            compute_diff(&previous, self.trees.get(&tree_id).expect("replaced tree"))
        } else {
            UiReflectionDiff {
                tree_id: tree_id.clone(),
                changed_nodes: self
                    .trees
                    .get(&tree_id)
                    .map(|tree| tree.nodes.keys().copied().collect())
                    .unwrap_or_default(),
                removed_nodes: Vec::new(),
            }
        };
        self.rebuild_node_index();
        if !diff.is_empty() {
            self.broadcast(UiNotification::ReflectionDiff(diff.clone()));
        }
        diff
    }

    pub fn query_tree(&self, tree_id: &UiTreeId) -> Option<UiReflectionSnapshot> {
        self.trees.get(tree_id).cloned()
    }

    pub fn query_node(&self, node_path: &UiNodePath) -> Option<UiNodeDescriptor> {
        let (tree_id, node_id) = self.node_index.get(node_path)?;
        self.trees
            .get(tree_id)
            .and_then(|tree| tree.nodes.get(node_id))
            .cloned()
    }

    pub fn query_property(
        &self,
        node_path: &UiNodePath,
        property_name: &str,
    ) -> Option<UiPropertyDescriptor> {
        self.query_node(node_path)
            .and_then(|node| node.properties.get(property_name).cloned())
    }

    pub fn set_property(
        &mut self,
        node_path: UiNodePath,
        property_name: String,
        value: Value,
    ) -> Result<(), UiInvocationError> {
        let (tree_id, node_id) = self
            .node_index
            .get(&node_path)
            .cloned()
            .ok_or_else(|| UiInvocationError::UnknownNode(node_path.0.clone()))?;
        let tree = self
            .trees
            .get_mut(&tree_id)
            .ok_or_else(|| UiInvocationError::UnknownTree(tree_id.0.clone()))?;
        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| UiInvocationError::UnknownNode(node_path.0.clone()))?;
        let property = node.properties.get_mut(&property_name).ok_or_else(|| {
            UiInvocationError::UnknownProperty {
                node_path: node_path.0.clone(),
                property_name: property_name.clone(),
            }
        })?;
        if !property.writable {
            return Err(UiInvocationError::PropertyNotWritable {
                node_path: node_path.0.clone(),
                property_name,
            });
        }
        property.reflected_value = value;
        let diff = UiReflectionDiff {
            tree_id,
            changed_nodes: vec![node_id],
            removed_nodes: Vec::new(),
        };
        self.broadcast(UiNotification::ReflectionDiff(diff));
        Ok(())
    }

    pub(crate) fn rebuild_node_index(&mut self) {
        self.node_index.clear();
        for (tree_id, tree) in &self.trees {
            for (node_id, node) in &tree.nodes {
                self.node_index
                    .insert(node.node_path.clone(), (tree_id.clone(), *node_id));
            }
        }
    }
}
