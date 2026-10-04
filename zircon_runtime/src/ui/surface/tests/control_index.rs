use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    template::{UiCompiledBindingGeneration, UiCompiledBindingProgram, UiCompiledControlId},
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
};

use super::{reference_hash, UiSurfaceControlIndex};

#[test]
fn control_index_directories_use_hash_lookup_with_stable_candidate_order() {
    let source = include_str!("../control_index.rs");
    let state = source
        .split("struct UiSurfaceControlIndexState")
        .nth(1)
        .and_then(|source| source.split("impl UiSurfaceControlIndexState").next())
        .expect("control index state");

    assert!(state.contains("nodes_by_control_id: HashMap<String, BTreeSet<UiNodeId>>"));
    assert!(state.contains("control_id_by_node: HashMap<UiNodeId, String>"));
    assert!(state.contains("reference_node_ids_by_hash: HashMap<u64, BTreeSet<UiNodeId>>"));
    assert!(state.contains("reference_hashes_by_node: HashMap<UiNodeId, UiNodeReferenceHashes>"));
    assert!(state.contains("compiled_control_ids_by_name: HashMap<String, usize>"));
    assert!(state.contains("BTreeSet<UiNodeId>"));
    assert!(source.contains("self.nodes_by_control_id.reserve(node_count);"));
    assert!(source.contains("self.reference_node_ids_by_hash"));
    assert!(source.contains(".reserve(node_count.saturating_mul(2));"));
}

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
