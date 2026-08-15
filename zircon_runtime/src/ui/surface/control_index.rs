use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiTree};

#[derive(Clone, Debug, Default)]
pub(crate) struct UiSurfaceControlIndex {
    state: RefCell<UiSurfaceControlIndexState>,
}

impl UiSurfaceControlIndex {
    pub(super) fn node_id(&self, tree: &UiTree, control_id: &str) -> Option<UiNodeId> {
        let mut state = self.state.borrow_mut();
        if !state.initialized {
            state.rebuild(tree);
        } else {
            for node_id in tree.pending_mutation_node_ids() {
                state.synchronize_node(tree, *node_id);
            }
        }
        let resolved = state
            .nodes_by_control_id
            .get(control_id)
            .and_then(|node_ids| node_ids.first().copied());
        if resolved.is_some_and(|node_id| !node_has_control_id(tree, node_id, control_id)) {
            state.rebuild(tree);
            return state
                .nodes_by_control_id
                .get(control_id)
                .and_then(|node_ids| node_ids.first().copied());
        }
        resolved
    }

    /// Popup trigger identities require an unambiguous owner; the compatibility lookup intentionally
    /// returns the smallest duplicate for compatibility with generic property routing.
    pub(super) fn unique_node_id(&self, tree: &UiTree, control_id: &str) -> Option<UiNodeId> {
        let mut state = self.state.borrow_mut();
        if !state.initialized {
            state.rebuild(tree);
        } else {
            for node_id in tree.pending_mutation_node_ids() {
                state.synchronize_node(tree, *node_id);
            }
        }
        let indexed = state
            .nodes_by_control_id
            .get(control_id)
            .and_then(|node_ids| {
                (node_ids.len() == 1).then(|| *node_ids.first().expect("one control id entry"))
            });
        let actual = unique_control_node_id(tree, control_id);
        if indexed != actual {
            state.rebuild(tree);
        }
        actual
    }

    /// Resolves an unambiguous control through the surface-owned incremental index.
    ///
    /// Pending mutations are synchronized incrementally. Surface-owned callers
    /// mutate through `UiTreeNodes`, so this remains O(changed controls) rather
    /// than re-scanning the full tree for every open popup during extraction.
    pub(crate) fn unique_node_id_for_surface(
        &self,
        tree: &UiTree,
        control_id: &str,
    ) -> Option<UiNodeId> {
        let mut state = self.state.borrow_mut();
        if !state.initialized {
            state.rebuild(tree);
        } else {
            for node_id in tree.pending_mutation_node_ids() {
                state.synchronize_node(tree, *node_id);
            }
        }
        state
            .nodes_by_control_id
            .get(control_id)
            .and_then(|node_ids| {
                (node_ids.len() == 1).then(|| *node_ids.first().expect("one control id entry"))
            })
            .filter(|node_id| node_has_control_id(tree, *node_id, control_id))
    }

    pub(super) fn synchronize_pending(&self, tree: &UiTree) {
        let mut state = self.state.borrow_mut();
        if !state.initialized {
            return;
        }
        for node_id in tree.pending_mutation_node_ids() {
            state.synchronize_node(tree, *node_id);
        }
    }
}

#[derive(Clone, Debug, Default)]
struct UiSurfaceControlIndexState {
    initialized: bool,
    nodes_by_control_id: BTreeMap<String, BTreeSet<UiNodeId>>,
    control_id_by_node: BTreeMap<UiNodeId, String>,
}

impl UiSurfaceControlIndexState {
    fn rebuild(&mut self, tree: &UiTree) {
        self.nodes_by_control_id.clear();
        self.control_id_by_node.clear();
        for (node_id, node) in &tree.nodes {
            let Some(control_id) = node_control_id(node) else {
                continue;
            };
            self.insert(*node_id, control_id.to_string());
        }
        self.initialized = true;
    }

    fn synchronize_node(&mut self, tree: &UiTree, node_id: UiNodeId) {
        self.remove(node_id);
        let Some(control_id) = tree.nodes.get(&node_id).and_then(node_control_id) else {
            return;
        };
        self.insert(node_id, control_id.to_string());
    }

    fn insert(&mut self, node_id: UiNodeId, control_id: String) {
        self.nodes_by_control_id
            .entry(control_id.clone())
            .or_default()
            .insert(node_id);
        self.control_id_by_node.insert(node_id, control_id);
    }

    fn remove(&mut self, node_id: UiNodeId) {
        let Some(control_id) = self.control_id_by_node.remove(&node_id) else {
            return;
        };
        let remove_control =
            self.nodes_by_control_id
                .get_mut(&control_id)
                .is_some_and(|node_ids| {
                    node_ids.remove(&node_id);
                    node_ids.is_empty()
                });
        if remove_control {
            self.nodes_by_control_id.remove(&control_id);
        }
    }
}

// This is a derived lookup cache; it does not contribute to surface value identity.
impl PartialEq for UiSurfaceControlIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn node_control_id(node: &zircon_runtime_interface::ui::tree::UiTreeNode) -> Option<&str> {
    node.template_metadata.as_ref()?.control_id.as_deref()
}

fn node_has_control_id(tree: &UiTree, node_id: UiNodeId, control_id: &str) -> bool {
    tree.nodes.get(&node_id).and_then(node_control_id) == Some(control_id)
}

fn unique_control_node_id(tree: &UiTree, control_id: &str) -> Option<UiNodeId> {
    let mut matches = tree.nodes.iter().filter_map(|(node_id, node)| {
        (node_control_id(node) == Some(control_id)).then_some(*node_id)
    });
    let node_id = matches.next()?;
    matches.next().is_none().then_some(node_id)
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
    };

    use super::UiSurfaceControlIndex;

    #[test]
    fn cached_control_lookup_revalidates_after_metadata_change() {
        let mut tree = UiTree::new(UiTreeId::new("control-index"));
        tree.insert_root(node(1, "Action"));
        tree.insert_root(node(2, "Other"));
        let index = UiSurfaceControlIndex::default();

        assert_eq!(index.node_id(&tree, "Action"), Some(UiNodeId::new(1)));
        tree.node_mut(UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("FormerAction".to_string());
        tree.node_mut(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("Action".to_string());

        assert_eq!(index.node_id(&tree, "Action"), Some(UiNodeId::new(2)));
    }

    #[test]
    fn pending_insert_preserves_smallest_duplicate_control_id_semantics() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-duplicate"));
        tree.insert_root(node(2, "Action"));
        tree.clear_pending_mutation_node_ids();
        let index = UiSurfaceControlIndex::default();
        assert_eq!(index.node_id(&tree, "Action"), Some(UiNodeId::new(2)));

        tree.insert_root(node(1, "Action"));

        assert_eq!(index.node_id(&tree, "Action"), Some(UiNodeId::new(1)));
        assert_eq!(index.unique_node_id(&tree, "Action"), None);
    }

    #[test]
    fn unique_control_lookup_rejects_duplicate_control_ids() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-unique"));
        tree.insert_root(node(1, "Action"));
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(1))
        );

        tree.insert_root(node(2, "Action"));

        assert_eq!(index.unique_node_id(&tree, "Action"), None);
    }

    #[test]
    fn surface_unique_lookup_tracks_incremental_duplicate_resolution() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-surface-unique"));
        tree.insert_root(node(1, "Action"));
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.unique_node_id_for_surface(&tree, "Action"),
            Some(UiNodeId::new(1))
        );

        tree.insert_root(node(2, "Action"));
        assert_eq!(index.unique_node_id_for_surface(&tree, "Action"), None);

        tree.node_mut(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("OtherAction".to_string());
        assert_eq!(
            index.unique_node_id_for_surface(&tree, "Action"),
            Some(UiNodeId::new(1))
        );
    }

    #[test]
    fn whole_tree_replacement_rebuilds_a_stale_cached_node() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-replacement"));
        tree.insert_root(node(1, "Action"));
        tree.clear_pending_mutation_node_ids();
        let index = UiSurfaceControlIndex::default();
        assert_eq!(index.node_id(&tree, "Action"), Some(UiNodeId::new(1)));

        let mut replacement = UiTree::new(UiTreeId::new("control-index-replacement"));
        replacement.insert_root(node(2, "Action"));
        replacement.clear_pending_mutation_node_ids();

        assert_eq!(
            index.node_id(&replacement, "Action"),
            Some(UiNodeId::new(2))
        );
    }

    #[test]
    fn unique_lookup_rejects_a_same_id_replacement_that_introduces_a_duplicate() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-unique-replacement"));
        tree.insert_root(node(1, "Action"));
        tree.clear_pending_mutation_node_ids();
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(1))
        );

        let mut replacement = UiTree::new(UiTreeId::new("control-index-unique-replacement"));
        replacement.insert_root(node(1, "Action"));
        replacement.insert_root(node(2, "Action"));
        replacement.clear_pending_mutation_node_ids();

        assert_eq!(index.unique_node_id(&replacement, "Action"), None);
    }

    #[test]
    fn pending_metadata_change_can_be_synchronized_before_dirty_clear() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-clear"));
        tree.insert_root(node(1, "Action"));
        let index = UiSurfaceControlIndex::default();
        assert_eq!(index.node_id(&tree, "Action"), Some(UiNodeId::new(1)));

        tree.node_mut(UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("RenamedAction".to_string());
        index.synchronize_pending(&tree);
        tree.clear_pending_mutation_node_ids();

        assert_eq!(index.node_id(&tree, "Action"), None);
        assert_eq!(
            index.node_id(&tree, "RenamedAction"),
            Some(UiNodeId::new(1))
        );
    }

    fn node(id: u64, control_id: &str) -> UiTreeNode {
        let mut node = UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("control/{id}")));
        node.template_metadata = Some(UiTemplateNodeMetadata {
            control_id: Some(control_id.to_string()),
            ..Default::default()
        });
        node
    }
}
