use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    template::{UiCompiledBindingGeneration, UiCompiledBindingProgram, UiCompiledControlId},
    tree::{UiTree, UiTreeNode},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct UiSurfaceControlIndex {
    state: RefCell<UiSurfaceControlIndexState>,
}

impl UiSurfaceControlIndex {
    pub(super) fn install_compiled_controls(
        &self,
        tree: &UiTree,
        program: &UiCompiledBindingProgram,
    ) {
        let mut state = self.state.borrow_mut();
        state.synchronize_pending(tree);
        state.install_compiled_controls(program);
    }

    /// Resolves a control only when its identity is unambiguous in the current tree.
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

    pub(crate) fn unique_node_id_for_compiled_control(
        &self,
        tree: &UiTree,
        program: &UiCompiledBindingProgram,
        control_id: UiCompiledControlId,
    ) -> Option<UiNodeId> {
        let control_name = program.control_name(control_id)?;
        let mut state = self.state.borrow_mut();
        state.synchronize_pending(tree);
        if state.compiled_generation != program.generation()
            || state.compiled_node_ids.len() != program.control_count()
        {
            state.install_compiled_controls(program);
        }
        state
            .compiled_node_ids
            .get(control_id.get() as usize)
            .copied()
            .flatten()
            .filter(|node_id| node_has_control_id(tree, *node_id, control_name))
    }

    /// Resolves a control id or node path through the surface-owned incremental index.
    ///
    /// Hash buckets avoid retaining a second copy of every node path. Candidates are
    /// validated against the live tree, so hash collisions cannot change identity.
    pub(crate) fn first_node_id_for_reference(
        &self,
        tree: &UiTree,
        reference: &str,
    ) -> Option<UiNodeId> {
        let mut state = self.state.borrow_mut();
        state.synchronize_pending(tree);
        let node_ids = state
            .reference_node_ids_by_hash
            .get(&reference_hash(reference))?;
        node_ids
            .iter()
            .copied()
            .find(|node_id| node_matches_reference(tree, *node_id, reference))
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
    nodes_by_control_id: HashMap<String, BTreeSet<UiNodeId>>,
    control_id_by_node: HashMap<UiNodeId, String>,
    reference_node_ids_by_hash: HashMap<u64, BTreeSet<UiNodeId>>,
    reference_hashes_by_node: HashMap<UiNodeId, UiNodeReferenceHashes>,
    compiled_generation: UiCompiledBindingGeneration,
    compiled_control_ids_by_name: HashMap<String, usize>,
    compiled_node_ids: Vec<Option<UiNodeId>>,
}

impl UiSurfaceControlIndexState {
    fn synchronize_pending(&mut self, tree: &UiTree) {
        if !self.initialized {
            self.rebuild(tree);
            return;
        }
        for node_id in tree.pending_mutation_node_ids() {
            self.synchronize_node(tree, *node_id);
        }
    }

    fn install_compiled_controls(&mut self, program: &UiCompiledBindingProgram) {
        self.compiled_generation = program.generation();
        self.compiled_control_ids_by_name.clear();
        self.compiled_control_ids_by_name
            .reserve(program.control_count());
        self.compiled_node_ids = vec![None; program.control_count()];
        for (index, control_name) in program.iter_control_names().enumerate() {
            self.compiled_control_ids_by_name
                .insert(control_name.to_string(), index);
            self.compiled_node_ids[index] =
                unique_indexed_node_id(self.nodes_by_control_id.get(control_name));
        }
    }

    fn rebuild(&mut self, tree: &UiTree) {
        let node_count = tree.nodes.len();
        self.nodes_by_control_id.clear();
        self.control_id_by_node.clear();
        self.reference_node_ids_by_hash.clear();
        self.reference_hashes_by_node.clear();
        self.nodes_by_control_id.reserve(node_count);
        self.control_id_by_node.reserve(node_count);
        self.reference_node_ids_by_hash
            .reserve(node_count.saturating_mul(2));
        self.reference_hashes_by_node.reserve(node_count);
        self.compiled_node_ids.fill(None);
        for (node_id, node) in &tree.nodes {
            self.insert(*node_id, node);
        }
        self.initialized = true;
    }

    fn synchronize_node(&mut self, tree: &UiTree, node_id: UiNodeId) {
        self.remove(node_id);
        let Some(node) = tree.nodes.get(&node_id) else {
            return;
        };
        self.insert(node_id, node);
    }

    fn insert(&mut self, node_id: UiNodeId, node: &UiTreeNode) {
        let node_path_hash = reference_hash(node.node_path.0.as_str());
        self.insert_reference(node_id, node_path_hash);
        let control_id_hash = node_control_id(node).map(|control_id| {
            let control_id_hash = reference_hash(control_id);
            self.insert_reference(node_id, control_id_hash);
            let control_id = control_id.to_string();
            self.nodes_by_control_id
                .entry(control_id.clone())
                .or_default()
                .insert(node_id);
            self.control_id_by_node.insert(node_id, control_id.clone());
            self.refresh_compiled_control(&control_id);
            control_id_hash
        });
        self.reference_hashes_by_node.insert(
            node_id,
            UiNodeReferenceHashes {
                node_path: node_path_hash,
                control_id: control_id_hash,
            },
        );
    }

    fn remove(&mut self, node_id: UiNodeId) {
        self.remove_references(node_id);
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
        self.refresh_compiled_control(&control_id);
    }

    fn insert_reference(&mut self, node_id: UiNodeId, hash: u64) {
        self.reference_node_ids_by_hash
            .entry(hash)
            .or_default()
            .insert(node_id);
    }

    fn remove_references(&mut self, node_id: UiNodeId) {
        let Some(hashes) = self.reference_hashes_by_node.remove(&node_id) else {
            return;
        };
        self.remove_reference(node_id, hashes.node_path);
        if hashes.control_id != Some(hashes.node_path) {
            if let Some(control_id_hash) = hashes.control_id {
                self.remove_reference(node_id, control_id_hash);
            }
        }
    }

    fn remove_reference(&mut self, node_id: UiNodeId, hash: u64) {
        let remove_bucket =
            self.reference_node_ids_by_hash
                .get_mut(&hash)
                .is_some_and(|node_ids| {
                    node_ids.remove(&node_id);
                    node_ids.is_empty()
                });
        if remove_bucket {
            self.reference_node_ids_by_hash.remove(&hash);
        }
    }

    fn refresh_compiled_control(&mut self, control_id: &str) {
        let Some(index) = self.compiled_control_ids_by_name.get(control_id).copied() else {
            return;
        };
        self.compiled_node_ids[index] =
            unique_indexed_node_id(self.nodes_by_control_id.get(control_id));
    }
}

#[derive(Clone, Copy, Debug)]
struct UiNodeReferenceHashes {
    node_path: u64,
    control_id: Option<u64>,
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

fn node_matches_reference(tree: &UiTree, node_id: UiNodeId, reference: &str) -> bool {
    tree.nodes.get(&node_id).is_some_and(|node| {
        node.node_path.0 == reference || node_control_id(node) == Some(reference)
    })
}

fn reference_hash(value: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    value
        .as_bytes()
        .iter()
        .fold(FNV_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        })
}

fn unique_indexed_node_id(node_ids: Option<&BTreeSet<UiNodeId>>) -> Option<UiNodeId> {
    node_ids.and_then(|node_ids| {
        (node_ids.len() == 1).then(|| *node_ids.first().expect("one control id entry"))
    })
}

fn unique_control_node_id(tree: &UiTree, control_id: &str) -> Option<UiNodeId> {
    let mut matches = tree.nodes.iter().filter_map(|(node_id, node)| {
        (node_control_id(node) == Some(control_id)).then_some(*node_id)
    });
    let node_id = matches.next()?;
    matches.next().is_none().then_some(node_id)
}

#[cfg(test)]
#[path = "tests/control_index.rs"]
mod tests;
