use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use serde_json;

use super::{mark_structure_dirty, UiTree};
use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use crate::ui::layout::{UiSlot, UiSlotKind};
use crate::ui::tree::{UiDirtyFlags, UiTreeNode};

#[test]
fn bulk_insert_assigns_dense_paint_order_without_rescanning_existing_nodes() {
    const NODE_COUNT: u64 = 10_000;
    let mut tree = UiTree::new(UiTreeId::new("paint-order.bulk"));

    for index in 0..NODE_COUNT {
        tree.insert_root(node(index));
    }

    assert_eq!(tree.nodes.len(), NODE_COUNT as usize);
    assert_eq!(tree.node(UiNodeId::new(0)).unwrap().paint_order, 0);
    assert_eq!(
        tree.node(UiNodeId::new(NODE_COUNT - 1))
            .unwrap()
            .paint_order,
        NODE_COUNT - 1
    );
    assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 0);
}

#[test]
fn bulk_child_insert_preserves_the_cursor_while_mutating_the_parent() {
    const CHILD_COUNT: u64 = 10_000;
    let mut tree = UiTree::new(UiTreeId::new("paint-order.children"));
    let root_id = UiNodeId::new(0);
    tree.insert_root(node(0));

    for index in 1..=CHILD_COUNT {
        tree.insert_child(root_id, node(index)).unwrap();
    }

    assert_eq!(
        tree.node(root_id).unwrap().children.len(),
        CHILD_COUNT as usize
    );
    assert_eq!(
        tree.node(UiNodeId::new(CHILD_COUNT)).unwrap().paint_order,
        CHILD_COUNT
    );
    assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 0);
}

#[test]
fn child_structure_changes_invalidate_the_parent_measurement_cache() {
    let root_id = UiNodeId::new(0);
    let mut tree = UiTree::new(UiTreeId::new("layout-cache.structure"));
    tree.insert_root(node(0));
    tree.node_mut(root_id)
        .expect("root")
        .layout_cache
        .complete_measure();
    tree.clear_pending_mutation_node_ids();

    tree.insert_child(root_id, node(1)).expect("child");

    assert!(!tree.node(root_id).expect("root").layout_cache.measure_valid);
}

#[test]
fn dirty_index_tracks_mutations_and_state_flags_without_idle_rescans() {
    let node_id = UiNodeId::new(0);
    let mut tree = UiTree::new(UiTreeId::new("dirty-index.incremental"));
    tree.insert_root(node(0));
    tree.nodes.clear_pending_mutation_node_ids();
    assert_eq!(tree.nodes.dirty_index_node_visits(), 0);

    tree.node_mut(node_id).expect("node").dirty = UiDirtyFlags::default();
    tree.nodes.clear_pending_mutation_node_ids();
    assert_eq!(tree.nodes.dirty_flags(), UiDirtyFlags::default());
    let idle_visits = tree.nodes.dirty_index_node_visits();
    assert_eq!(idle_visits, 1);
    assert_eq!(tree.nodes.dirty_node_count(), 0);
    assert!(tree.nodes.dirty_node_ids().is_empty());
    assert_eq!(tree.nodes.dirty_index_node_visits(), idle_visits);

    tree.node_mut(node_id).expect("node").dirty.render = true;
    assert_eq!(tree.nodes.dirty_node_count(), 1);
    assert_eq!(tree.nodes.dirty_node_ids(), BTreeSet::from([node_id]));
    assert!(tree.nodes.dirty_node_flags(&node_id).render);
    assert_eq!(
        tree.nodes.dirty_index_node_visits(),
        idle_visits.saturating_add(1)
    );

    tree.node_mut(node_id).expect("node").dirty = UiDirtyFlags::default();
    tree.node_mut(node_id).expect("node").state_flags.dirty = true;
    let state_dirty = tree.nodes.dirty_flags();
    assert!(state_dirty.hit_test && state_dirty.render && state_dirty.input);
    assert!(!state_dirty.layout);
}

#[test]
fn deserialized_dirty_index_builds_once_then_refreshes_pending_nodes() {
    let node_id = UiNodeId::new(0);
    let mut original = UiTree::new(UiTreeId::new("dirty-index.deserialize"));
    original.insert_root(node(0));
    original.node_mut(node_id).expect("node").dirty = UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    };
    original.clear_pending_mutation_node_ids();

    let serialized = serde_json::to_vec(&original).expect("serialize tree");
    let mut restored: UiTree = serde_json::from_slice(&serialized).expect("deserialize tree");

    assert_eq!(restored.nodes.dirty_node_count(), 1);
    let full_build_visits = restored.nodes.dirty_index_node_visits();
    assert_eq!(full_build_visits, 1);
    assert!(restored.nodes.dirty_flags().render);
    assert_eq!(restored.nodes.dirty_index_node_visits(), full_build_visits);

    restored.node_mut(node_id).expect("node").dirty = UiDirtyFlags::default();
    assert_eq!(restored.nodes.dirty_node_count(), 0);
    assert_eq!(
        restored.nodes.dirty_index_node_visits(),
        full_build_visits.saturating_add(1)
    );
}

#[test]
fn bulk_mutation_tracks_all_nodes_without_an_intermediate_key_snapshot() {
    let source = include_str!("../ui_tree.rs");
    let track_all_nodes = source
        .split_once("    fn track_all_nodes(&mut self) {")
        .and_then(|(_, remainder)| remainder.split_once("    fn get_mut_preserving_paint_order"))
        .map(|(body, _)| body)
        .expect("track_all_nodes implementation");
    assert!(track_all_nodes.contains("for node_id in self.nodes.keys().copied()"));
    assert!(!track_all_nodes.contains("collect::<BTreeSet<_>>()"));

    let mut tree = UiTree::new(UiTreeId::new("dirty-index.bulk"));
    tree.insert_root(node(0));
    tree.insert_root(node(1));
    // Insertion marks each new node dirty; establish an idle baseline before testing bulk mutation.
    for current in tree.nodes.values_mut() {
        current.dirty = Default::default();
    }
    tree.nodes.clear_pending_mutation_node_ids();
    assert_eq!(tree.nodes.dirty_node_count(), 0);

    for current in tree.nodes.values_mut() {
        current.dirty.text = true;
    }

    assert_eq!(tree.nodes.dirty_node_count(), 2);
    assert!(tree.nodes.dirty_flags().text);
    assert_eq!(
        tree.nodes.dirty_node_ids(),
        BTreeSet::from([UiNodeId::new(0), UiNodeId::new(1)])
    );
}

#[test]
fn dirty_node_id_extension_merges_into_existing_destination() {
    let node_id = UiNodeId::new(0);
    let mut tree = UiTree::new(UiTreeId::new("dirty-index.extension"));
    tree.insert_root(node(0));
    tree.clear_pending_mutation_node_ids();
    tree.node_mut(node_id).expect("node").dirty.render = true;

    let mut node_ids = BTreeSet::from([UiNodeId::new(99)]);
    tree.nodes.extend_dirty_node_ids(&mut node_ids);

    assert_eq!(
        node_ids,
        BTreeSet::from([UiNodeId::new(0), UiNodeId::new(99)])
    );
    assert!(include_str!("../ui_tree.rs").contains("pub fn extend_dirty_node_ids"));
}

#[test]
fn dirty_node_entry_extension_writes_directly_into_existing_destination() {
    let node_id = UiNodeId::new(0);
    let mut tree = UiTree::new(UiTreeId::new("dirty-index.entry-extension"));
    tree.insert_root(node(0));
    tree.clear_pending_mutation_node_ids();
    tree.node_mut(node_id).expect("node").dirty = UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    };

    let mut entries = Vec::with_capacity(3);
    entries.push((UiNodeId::new(99), UiDirtyFlags::default()));
    let entries_ptr = entries.as_ptr();
    tree.nodes.extend_dirty_node_entries(&mut entries);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries.as_ptr(), entries_ptr);
    assert_eq!(entries[0].0, UiNodeId::new(99));
    assert_eq!(entries[1].0, node_id);
    assert!(entries[1].1.render);
    assert!(include_str!("../ui_tree.rs").contains("pub fn extend_dirty_node_entries"));
}

#[test]
fn layout_order_generation_ignores_non_order_slot_mutations() {
    let root_id = UiNodeId::new(0);
    let child_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout-order.generation"));
    tree.insert_root(node(0));
    tree.insert_child(root_id, node(1)).expect("child");
    tree.push_layout_slot(UiSlot::new(root_id, child_id, UiSlotKind::Free));
    tree.clear_pending_mutation_node_ids();
    let stable_generation = tree.layout_order_generation();

    tree.mutate_layout_slot(0, |slot| slot.z_order = 7)
        .expect("mutate non-order slot field");

    assert_eq!(tree.layout_order_generation(), stable_generation);
    assert!(tree.pending_layout_order_parent_ids().is_empty());

    tree.mutate_layout_slot(0, |slot| slot.order = 2)
        .expect("mutate slot order");

    assert_ne!(tree.layout_order_generation(), stable_generation);
    assert_eq!(
        tree.pending_layout_order_parent_ids(),
        &BTreeSet::from([root_id])
    );
}

#[test]
fn deserialized_layout_slot_authority_rebuilds_once_and_keeps_missing_edges_authoritative() {
    let parent_id = UiNodeId::new(0);
    let child_id = UiNodeId::new(1);
    let mut original = UiTree::new(UiTreeId::new("layout-slot.deserialize"));
    original.insert_root(node(0));
    original.insert_child(parent_id, node(1)).expect("child");
    original.push_layout_slot(UiSlot::new(parent_id, child_id, UiSlotKind::Linear));
    let serialized = serde_json::to_vec(&original).expect("serialize UI tree");
    let restored: UiTree = serde_json::from_slice(&serialized).expect("deserialize UI tree");

    assert_eq!(restored.layout_slot_authority_rebuild_count(), 0);
    assert_eq!(
        restored.layout_slot_index_for_edge_kind(parent_id, child_id, UiSlotKind::Linear),
        Some(0)
    );
    assert_eq!(restored.layout_slot_authority_rebuild_count(), 1);
    for missing_child in 2..=1_000 {
        assert_eq!(
            restored.layout_slot_index_for_edge_kind(
                parent_id,
                UiNodeId::new(missing_child),
                UiSlotKind::Linear,
            ),
            None
        );
    }
    assert_eq!(restored.layout_slot_authority_rebuild_count(), 1);
}

#[test]
fn same_cardinality_slot_rebind_updates_the_edge_authority_without_rebuilding() {
    let parent_id = UiNodeId::new(0);
    let first_child_id = UiNodeId::new(1);
    let next_child_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("layout-slot.rebind"));
    tree.push_layout_slot(UiSlot::new(parent_id, first_child_id, UiSlotKind::Linear));
    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, first_child_id, UiSlotKind::Linear,),
        Some(0)
    );
    assert_eq!(tree.layout_slot_authority_rebuild_count(), 1);

    tree.mutate_layout_slot(0, |slot| slot.child_id = next_child_id)
        .expect("rebind slot");

    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, first_child_id, UiSlotKind::Linear,),
        None
    );
    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, next_child_id, UiSlotKind::Linear),
        Some(0)
    );
    assert_eq!(tree.layout_slot_authority_rebuild_count(), 1);
}

#[test]
fn slot_rebind_preserves_flat_slot_precedence_on_an_existing_edge() {
    let parent_id = UiNodeId::new(0);
    let first_child_id = UiNodeId::new(1);
    let next_child_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("layout-slot.rebind-order"));
    tree.push_layout_slot(UiSlot::new(parent_id, first_child_id, UiSlotKind::Linear));
    tree.push_layout_slot(UiSlot::new(parent_id, next_child_id, UiSlotKind::Linear));
    assert_eq!(
        tree.first_layout_slot_index_for_edge(parent_id, next_child_id),
        Some(1)
    );

    tree.mutate_layout_slot(0, |slot| slot.child_id = next_child_id)
        .expect("rebind slot");

    assert_eq!(
        tree.first_layout_slot_index_for_edge(parent_id, next_child_id),
        Some(0)
    );
    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, next_child_id, UiSlotKind::Linear),
        Some(0)
    );
    assert_eq!(
        tree.first_layout_slot_index_for_edge(parent_id, first_child_id),
        None
    );
    assert_eq!(tree.layout_slot_authority_rebuild_count(), 1);
}

#[test]
fn bulk_slot_retention_reindexes_once_and_removes_the_retired_edge() {
    let parent_id = UiNodeId::new(0);
    let retained_child_id = UiNodeId::new(1);
    let removed_child_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("layout-slot.retain"));
    tree.push_layout_slot(UiSlot::new(
        parent_id,
        retained_child_id,
        UiSlotKind::Linear,
    ));
    tree.push_layout_slot(UiSlot::new(parent_id, removed_child_id, UiSlotKind::Linear));
    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, removed_child_id, UiSlotKind::Linear,),
        Some(1)
    );

    tree.retain_layout_slots(|slot| slot.child_id != removed_child_id);

    assert_eq!(tree.layout_slot_authority_rebuild_count(), 2);
    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, retained_child_id, UiSlotKind::Linear,),
        Some(0)
    );
    assert_eq!(
        tree.layout_slot_index_for_edge_kind(parent_id, removed_child_id, UiSlotKind::Linear,),
        None
    );

    tree.retain_layout_slots(|_| true);
    assert_eq!(tree.layout_slot_authority_rebuild_count(), 2);
}

#[test]
fn deserialized_tree_rebuilds_paint_order_cursor_only_once() {
    const EXISTING_NODE_COUNT: u64 = 4_096;
    let mut original = UiTree::new(UiTreeId::new("paint-order.deserialize"));
    for index in 0..EXISTING_NODE_COUNT {
        original.insert_root(node(index));
    }
    let serialized = serde_json::to_vec(&original).expect("serialize UI tree");
    let mut restored: UiTree = serde_json::from_slice(&serialized).expect("deserialize UI tree");

    restored.insert_root(node(EXISTING_NODE_COUNT));
    restored.insert_root(node(EXISTING_NODE_COUNT + 1));

    assert_eq!(
        restored
            .node(UiNodeId::new(EXISTING_NODE_COUNT))
            .unwrap()
            .paint_order,
        EXISTING_NODE_COUNT
    );
    assert_eq!(
        restored
            .node(UiNodeId::new(EXISTING_NODE_COUNT + 1))
            .unwrap()
            .paint_order,
        EXISTING_NODE_COUNT + 1
    );
    assert_eq!(
        restored.nodes.paint_order_cursor_rebuild_node_visits(),
        EXISTING_NODE_COUNT as usize
    );
}

#[test]
fn mutable_node_access_invalidates_the_paint_order_cursor() {
    let mut tree = UiTree::new(UiTreeId::new("paint-order.mutation"));
    for index in 0..3 {
        tree.insert_root(node(index));
    }
    tree.node_mut(UiNodeId::new(1)).unwrap().paint_order = 40;

    tree.insert_root(node(3));
    tree.insert_root(node(4));

    assert_eq!(tree.node(UiNodeId::new(3)).unwrap().paint_order, 41);
    assert_eq!(tree.node(UiNodeId::new(4)).unwrap().paint_order, 42);
    assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 3);
}

#[test]
fn cursor_rebuild_does_not_reuse_a_retired_high_water_order() {
    let mut tree = UiTree::new(UiTreeId::new("paint-order.retired"));
    tree.insert_root(node(0));
    tree.insert_root(node(1));
    tree.node_mut(UiNodeId::new(0)).unwrap().dirty.layout = false;
    tree.nodes.remove(&UiNodeId::new(1));

    tree.insert_root(node(2));

    assert_eq!(tree.node(UiNodeId::new(2)).unwrap().paint_order, 2);
    assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 1);
}

#[test]
fn clearing_nodes_does_not_reuse_a_retired_node_incarnation() {
    let mut tree = UiTree::new(UiTreeId::new("node-incarnation.clear"));
    tree.insert_root(node(0));
    let retired = tree.node_incarnation(UiNodeId::new(0)).unwrap();

    tree.roots.clear();
    tree.nodes.clear();
    tree.insert_root(node(0));

    assert!(tree.node_incarnation(UiNodeId::new(0)).unwrap() > retired);
}

#[test]
#[ignore = "release-only paint-order performance evidence"]
fn paint_order_cursor_release_benchmark_evidence() {
    const NODE_COUNT: u64 = 10_000;
    const SAMPLE_PAIRS: usize = 21;
    let mut legacy_micros = Vec::with_capacity(SAMPLE_PAIRS);
    let mut cursor_micros = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            let mut legacy_nodes = BTreeMap::<UiNodeId, UiTreeNode>::new();
            let mut legacy_roots = Vec::with_capacity(NODE_COUNT as usize);
            for index in 0..NODE_COUNT {
                let paint_order = legacy_nodes
                    .values()
                    .map(|node| node.paint_order)
                    .max()
                    .map_or(0, |paint_order| paint_order.saturating_add(1));
                let mut node = node(index);
                node.paint_order = paint_order;
                mark_structure_dirty(&mut node);
                legacy_roots.push(node.node_id);
                legacy_nodes.insert(node.node_id, node);
            }
            black_box((&legacy_nodes, &legacy_roots));
            legacy_micros.push(started.elapsed().as_micros());
        };
        let mut measure_cursor = || {
            let started = Instant::now();
            let mut tree = UiTree::new(UiTreeId::new("paint-order.benchmark"));
            for index in 0..NODE_COUNT {
                tree.insert_root(node(index));
            }
            black_box(&tree);
            cursor_micros.push(started.elapsed().as_micros());
            assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 0);
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_cursor();
        } else {
            measure_cursor();
            measure_legacy();
        }
    }

    let legacy_csv = legacy_micros
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let cursor_csv = cursor_micros
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let legacy_p95_us = nearest_rank_percentile(&legacy_micros, 95);
    let cursor_p95_us = nearest_rank_percentile(&cursor_micros, 95);
    println!(
        "UI_TREE_PAINT_ORDER_BENCH_V1 node_count={NODE_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_scan_visits=49995000 cursor_scan_visits=0 legacy_p95_us={legacy_p95_us} cursor_p95_us={cursor_p95_us} legacy_us={legacy_csv} cursor_us={cursor_csv}"
    );
    assert!(
        cursor_p95_us.saturating_mul(4) <= legacy_p95_us,
        "cursor P95 {cursor_p95_us}us must be at most 25% of legacy P95 {legacy_p95_us}us"
    );
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn node(index: u64) -> UiTreeNode {
    UiTreeNode::new(
        UiNodeId::new(index),
        UiNodePath::new(format!("root/{index}")),
    )
}
