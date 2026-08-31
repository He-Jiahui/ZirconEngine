use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
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
    nodes_by_control_id: BTreeMap<String, BTreeSet<UiNodeId>>,
    control_id_by_node: BTreeMap<UiNodeId, String>,
    reference_node_ids_by_hash: BTreeMap<u64, BTreeSet<UiNodeId>>,
    reference_hashes_by_node: BTreeMap<UiNodeId, UiNodeReferenceHashes>,
    compiled_generation: UiCompiledBindingGeneration,
    compiled_control_ids_by_name: BTreeMap<String, usize>,
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
        self.compiled_node_ids = vec![None; program.control_count()];
        for (index, control_name) in program.iter_control_names().enumerate() {
            self.compiled_control_ids_by_name
                .insert(control_name.to_string(), index);
            self.compiled_node_ids[index] =
                unique_indexed_node_id(self.nodes_by_control_id.get(control_name));
        }
    }

    fn rebuild(&mut self, tree: &UiTree) {
        self.nodes_by_control_id.clear();
        self.control_id_by_node.clear();
        self.reference_node_ids_by_hash.clear();
        self.reference_hashes_by_node.clear();
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
mod tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        template::{UiCompiledBindingGeneration, UiCompiledBindingProgram, UiCompiledControlId},
        tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
    };

    use super::{UiSurfaceControlIndex, reference_hash};

    #[test]
    fn cached_control_lookup_revalidates_after_metadata_change() {
        let mut tree = UiTree::new(UiTreeId::new("control-index"));
        tree.insert_root(node(1, "Action"));
        tree.insert_root(node(2, "Other"));
        let index = UiSurfaceControlIndex::default();

        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(1))
        );
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

        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(2))
        );
    }

    #[test]
    fn pending_insert_rejects_duplicate_control_ids() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-duplicate"));
        tree.insert_root(node(2, "Action"));
        tree.clear_pending_mutation_node_ids();
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(2))
        );

        tree.insert_root(node(1, "Action"));

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
    fn reference_lookup_preserves_tree_order_across_control_and_path_matches() {
        let mut tree = UiTree::new(UiTreeId::new("reference-index-order"));
        tree.insert_root(node_with_path(9, "Other", "Action"));
        tree.insert_root(node_with_path(3, "Action", "other/path"));
        let index = UiSurfaceControlIndex::default();

        assert_eq!(
            index.first_node_id_for_reference(&tree, "Action"),
            Some(UiNodeId::new(3))
        );
    }

    #[test]
    fn reference_lookup_tracks_pending_control_and_path_changes() {
        let mut tree = UiTree::new(UiTreeId::new("reference-index-pending"));
        tree.insert_root(node_with_path(1, "Action", "old/path"));
        tree.insert_root(node_with_path(2, "Other", "other/path"));
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.first_node_id_for_reference(&tree, "Action"),
            Some(UiNodeId::new(1))
        );

        let first = tree.node_mut(UiNodeId::new(1)).unwrap();
        first.node_path = UiNodePath::new("former/path");
        first.template_metadata.as_mut().unwrap().control_id = Some("FormerAction".to_string());
        let second = tree.node_mut(UiNodeId::new(2)).unwrap();
        second.node_path = UiNodePath::new("Action");

        assert_eq!(
            index.first_node_id_for_reference(&tree, "Action"),
            Some(UiNodeId::new(2))
        );
        assert_eq!(
            index.first_node_id_for_reference(&tree, "FormerAction"),
            Some(UiNodeId::new(1))
        );
        assert_eq!(index.first_node_id_for_reference(&tree, "old/path"), None);
    }

    #[test]
    fn reference_lookup_rejects_a_hash_bucket_false_positive() {
        let mut tree = UiTree::new(UiTreeId::new("reference-index-collision"));
        tree.insert_root(node_with_path(1, "Other", "unrelated/path"));
        tree.insert_root(node_with_path(2, "Action", "target/path"));
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.first_node_id_for_reference(&tree, "Action"),
            Some(UiNodeId::new(2))
        );

        index
            .state
            .borrow_mut()
            .reference_node_ids_by_hash
            .entry(reference_hash("Action"))
            .or_default()
            .insert(UiNodeId::new(1));

        assert_eq!(
            index.first_node_id_for_reference(&tree, "Action"),
            Some(UiNodeId::new(2))
        );
    }

    #[test]
    fn whole_tree_replacement_rebuilds_a_stale_cached_node() {
        let mut tree = UiTree::new(UiTreeId::new("control-index-replacement"));
        tree.insert_root(node(1, "Action"));
        tree.clear_pending_mutation_node_ids();
        let index = UiSurfaceControlIndex::default();
        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(1))
        );

        let mut replacement = UiTree::new(UiTreeId::new("control-index-replacement"));
        replacement.insert_root(node(2, "Action"));
        replacement.clear_pending_mutation_node_ids();

        assert_eq!(
            index.unique_node_id(&replacement, "Action"),
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
        assert_eq!(
            index.unique_node_id(&tree, "Action"),
            Some(UiNodeId::new(1))
        );

        tree.node_mut(UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("RenamedAction".to_string());
        index.synchronize_pending(&tree);
        tree.clear_pending_mutation_node_ids();

        assert_eq!(index.unique_node_id(&tree, "Action"), None);
        assert_eq!(
            index.unique_node_id(&tree, "RenamedAction"),
            Some(UiNodeId::new(1))
        );
    }

    #[test]
    fn compiled_control_slots_track_incremental_duplicates_and_generation_changes() {
        let mut tree = UiTree::new(UiTreeId::new("compiled-control-slots"));
        tree.insert_root(node(1, "Action"));
        tree.insert_root(node(2, "Other"));
        let index = UiSurfaceControlIndex::default();
        let action_program = compiled_program(1, ["Action"]);
        index.install_compiled_controls(&tree, &action_program);
        tree.clear_pending_mutation_node_ids();

        assert_eq!(
            index.unique_node_id_for_compiled_control(
                &tree,
                &action_program,
                UiCompiledControlId::new(0),
            ),
            Some(UiNodeId::new(1))
        );

        tree.node_mut(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("Action".to_string());
        assert_eq!(
            index.unique_node_id_for_compiled_control(
                &tree,
                &action_program,
                UiCompiledControlId::new(0),
            ),
            None
        );

        tree.node_mut(UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .control_id = Some("FormerAction".to_string());
        assert_eq!(
            index.unique_node_id_for_compiled_control(
                &tree,
                &action_program,
                UiCompiledControlId::new(0),
            ),
            Some(UiNodeId::new(2))
        );

        let other_program = compiled_program(2, ["Other"]);
        assert_eq!(
            index.unique_node_id_for_compiled_control(
                &tree,
                &other_program,
                UiCompiledControlId::new(0),
            ),
            None
        );
    }

    #[test]
    #[ignore = "release-only compiled control slot performance evidence"]
    fn compiled_control_dense_slot_p95_beats_string_index_lookup() {
        const CONTROL_COUNT: usize = 2_048;
        const LOOKUPS_PER_SAMPLE: usize = 8_192;
        const SAMPLE_PAIRS: usize = 21;

        let mut tree = UiTree::new(UiTreeId::new("compiled-control-slot-benchmark"));
        let names = (0..CONTROL_COUNT)
            .map(|index| format!("Control{index:04}"))
            .collect::<Vec<_>>();
        for (index, name) in names.iter().enumerate() {
            tree.insert_root(node(index as u64 + 1, name));
        }
        let program = UiCompiledBindingProgram::new(
            UiCompiledBindingGeneration::new(1),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            names.clone(),
            Vec::new(),
            Vec::new(),
        );
        let index = UiSurfaceControlIndex::default();
        index.install_compiled_controls(&tree, &program);
        tree.clear_pending_mutation_node_ids();

        let _ = sample_control_lookups(&index, &tree, &program, &names, LOOKUPS_PER_SAMPLE, true);
        let _ = sample_control_lookups(&index, &tree, &program, &names, LOOKUPS_PER_SAMPLE, false);

        let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples_us.push(sample_control_lookups(
                    &index,
                    &tree,
                    &program,
                    &names,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
                optimized_samples_us.push(sample_control_lookups(
                    &index,
                    &tree,
                    &program,
                    &names,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples_us.push(sample_control_lookups(
                    &index,
                    &tree,
                    &program,
                    &names,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
                legacy_samples_us.push(sample_control_lookups(
                    &index,
                    &tree,
                    &program,
                    &names,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
        let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
        assert!(
            optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(75),
            "compiled control slot P95 {optimized_p95_us}us must improve string index P95 {legacy_p95_us}us by at least 25%"
        );
        println!(
            "PERF-RUNTIME74-COMPILED-CONTROL-SLOT sample_pairs={SAMPLE_PAIRS} control_count={CONTROL_COUNT} lookups_per_sample={LOOKUPS_PER_SAMPLE} pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_string_index_lookups_per_sample={LOOKUPS_PER_SAMPLE} optimized_string_index_lookups_per_sample=0 string_lookup_reduction_percent=100 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=25",
            joined_samples(&legacy_samples_us),
            joined_samples(&optimized_samples_us),
        );
    }

    fn compiled_program<const N: usize>(
        generation: u64,
        controls: [&str; N],
    ) -> UiCompiledBindingProgram {
        UiCompiledBindingProgram::new(
            UiCompiledBindingGeneration::new(generation),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            controls.into_iter().map(str::to_string).collect(),
            Vec::new(),
            Vec::new(),
        )
    }

    fn sample_control_lookups(
        index: &UiSurfaceControlIndex,
        tree: &UiTree,
        program: &UiCompiledBindingProgram,
        names: &[String],
        lookups: usize,
        legacy: bool,
    ) -> u128 {
        let started = Instant::now();
        for lookup in 0..lookups {
            let control_index = lookup % names.len();
            let node_id = if legacy {
                index.unique_node_id_for_surface(tree, &names[control_index])
            } else {
                index.unique_node_id_for_compiled_control(
                    tree,
                    program,
                    UiCompiledControlId::new(control_index as u32),
                )
            };
            black_box(node_id.expect("benchmark control should remain unique"));
        }
        started.elapsed().as_micros().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(95).div_ceil(100).max(1);
        sorted[rank - 1]
    }

    fn joined_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn node(id: u64, control_id: &str) -> UiTreeNode {
        node_with_path(id, control_id, format!("control/{id}"))
    }

    fn node_with_path(id: u64, control_id: &str, path: impl Into<String>) -> UiTreeNode {
        let mut node = UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(path));
        node.template_metadata = Some(UiTemplateNodeMetadata {
            control_id: Some(control_id.to_string()),
            ..Default::default()
        });
        node
    }
}
