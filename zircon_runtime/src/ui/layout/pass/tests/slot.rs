use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{Anchor, StretchMode, UiContainerKind, UiSlot, UiSlotKind},
    tree::{UiTree, UiTreeNode},
};

use super::{slot_for_container_child, UiLayoutSlotIndex};

#[test]
fn indexed_lookup_preserves_first_matching_slot_semantics() {
    let parent_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let mut tree = UiTree::default();
    tree.replace_layout_slots(vec![
        UiSlot::new(parent_id, child_id, UiSlotKind::Free).with_order(9),
        UiSlot::new(parent_id, child_id, UiSlotKind::Linear).with_order(1),
        UiSlot::new(parent_id, child_id, UiSlotKind::Linear).with_order(2),
    ]);
    let slot_index = UiLayoutSlotIndex::for_tree(&tree);

    let slot = slot_for_container_child(
        &tree,
        &slot_index,
        parent_id,
        child_id,
        UiContainerKind::HorizontalBox(Default::default()),
    )
    .expect("linear slot should be indexed");

    assert_eq!(slot.order, 1);
}

#[test]
fn indexed_lookup_repairs_same_cardinality_edge_replacement() {
    let parent_id = UiNodeId::new(1);
    let old_child_id = UiNodeId::new(2);
    let next_child_id = UiNodeId::new(3);
    let mut tree = UiTree::default();
    tree.replace_layout_slots(vec![UiSlot::new(
        parent_id,
        old_child_id,
        UiSlotKind::Linear,
    )]);
    let slot_index = UiLayoutSlotIndex::for_tree(&tree);

    tree.replace_layout_slots(vec![UiSlot::new(
        parent_id,
        next_child_id,
        UiSlotKind::Linear,
    )
    .with_order(7)]);

    let slot = slot_for_container_child(
        &tree,
        &slot_index,
        parent_id,
        next_child_id,
        UiContainerKind::HorizontalBox(Default::default()),
    )
    .expect("replacement edge should repair the cached slot lookup");
    assert_eq!(slot.order, 7);
    assert!(slot_for_container_child(
        &tree,
        &slot_index,
        parent_id,
        old_child_id,
        UiContainerKind::HorizontalBox(Default::default()),
    )
    .is_none());
}

#[test]
fn ordered_children_reuse_one_generation_and_patch_same_cardinality_order_changes() {
    let parent_id = UiNodeId::new(1);
    let first_child_id = UiNodeId::new(2);
    let second_child_id = UiNodeId::new(3);
    let container = UiContainerKind::HorizontalBox(Default::default());
    let mut tree = UiTree::new(UiTreeId::new("layout.order.generation"));
    tree.insert_root(UiTreeNode::new(parent_id, UiNodePath::new("root")).with_container(container));
    tree.insert_child(
        parent_id,
        UiTreeNode::new(first_child_id, UiNodePath::new("root.first")),
    )
    .expect("insert first child");
    tree.insert_child(
        parent_id,
        UiTreeNode::new(second_child_id, UiNodePath::new("root.second")),
    )
    .expect("insert second child");
    tree.push_layout_slot(UiSlot::new(parent_id, first_child_id, UiSlotKind::Linear).with_order(1));
    tree.push_layout_slot(
        UiSlot::new(parent_id, second_child_id, UiSlotKind::Linear).with_order(0),
    );
    let slot_index = UiLayoutSlotIndex::for_tree(&tree);

    let first = slot_index.ordered_children_for_container(&tree, parent_id, container);
    let stable = slot_index.ordered_children_for_container(&tree, parent_id, container);
    assert_eq!(first.as_ref(), &[second_child_id, first_child_id]);
    assert!(Arc::ptr_eq(&first, &stable));

    tree.mutate_layout_slot(0, |slot| slot.order = -1)
        .expect("mutate first slot order");
    let changed = slot_index.ordered_children_for_container(&tree, parent_id, container);
    let changed_stable = slot_index.ordered_children_for_container(&tree, parent_id, container);

    assert_eq!(changed.as_ref(), &[first_child_id, second_child_id]);
    assert!(!Arc::ptr_eq(&first, &changed));
    assert!(Arc::ptr_eq(&changed, &changed_stable));
}

#[test]
fn ordered_children_patch_same_cardinality_public_child_reordering() {
    let parent_id = UiNodeId::new(1);
    let first_child_id = UiNodeId::new(2);
    let second_child_id = UiNodeId::new(3);
    let container = UiContainerKind::ScrollableBox(Default::default());
    let mut tree = UiTree::new(UiTreeId::new("layout.order.defensive-repair"));
    tree.insert_root(UiTreeNode::new(parent_id, UiNodePath::new("root")).with_container(container));
    tree.insert_child(
        parent_id,
        UiTreeNode::new(first_child_id, UiNodePath::new("root.first")),
    )
    .expect("insert first child");
    tree.insert_child(
        parent_id,
        UiTreeNode::new(second_child_id, UiNodePath::new("root.second")),
    )
    .expect("insert second child");
    let slot_index = UiLayoutSlotIndex::for_tree(&tree);
    let first = slot_index.ordered_children_for_container(&tree, parent_id, container);

    tree.node_mut(parent_id)
        .expect("parent")
        .children
        .swap(0, 1);
    slot_index.synchronize_ordered_children(&tree, &BTreeSet::from([parent_id]));
    let changed = slot_index.ordered_children_for_container(&tree, parent_id, container);

    assert_eq!(first.as_ref(), &[first_child_id, second_child_id]);
    assert_eq!(changed.as_ref(), &[second_child_id, first_child_id]);
    assert!(!Arc::ptr_eq(&first, &changed));
}

#[test]
fn one_child_dependency_change_patches_only_that_parent_membership() {
    const CHILD_COUNT: u64 = 1_000;
    let parent_id = UiNodeId::new(1);
    let changed_child_id = UiNodeId::new(778);
    let mut tree = UiTree::new(UiTreeId::new("layout.dependencies.exact-child"));
    tree.insert_root(
        UiTreeNode::new(parent_id, UiNodePath::new("root")).with_container(UiContainerKind::Free),
    );
    for child_index in 0..CHILD_COUNT {
        let child_id = UiNodeId::new(child_index + 2);
        let mut child = UiTreeNode::new(
            child_id,
            UiNodePath::new(format!("root.child-{child_index}")),
        );
        child.constraints.width.stretch_mode = StretchMode::Fixed;
        child.constraints.height.stretch_mode = StretchMode::Fixed;
        tree.insert_child(parent_id, child).expect("insert child");
    }
    let slot_index = UiLayoutSlotIndex::for_tree(&tree);
    tree.clear_pending_mutation_node_ids();
    let initial_evaluations = slot_index.parent_size_dependency_evaluations();

    tree.node_mut(changed_child_id)
        .expect("changed child")
        .anchor = Anchor::new(0.5, 0.0);
    slot_index.synchronize_parent_size_dependencies(&tree, &BTreeSet::from([changed_child_id]));

    assert_eq!(
        slot_index.parent_size_dependency_evaluations() - initial_evaluations,
        1
    );
    let mut dependent_children = Vec::new();
    slot_index.copy_parent_size_dependent_children(&tree, parent_id, &mut dependent_children);
    assert_eq!(dependent_children, vec![changed_child_id]);

    let before_removal = slot_index.parent_size_dependency_evaluations();
    tree.node_mut(changed_child_id)
        .expect("changed child")
        .anchor = Anchor::default();
    slot_index.synchronize_parent_size_dependencies(&tree, &BTreeSet::from([changed_child_id]));

    assert_eq!(
        slot_index.parent_size_dependency_evaluations() - before_removal,
        1
    );
    slot_index.copy_parent_size_dependent_children(&tree, parent_id, &mut dependent_children);
    assert!(dependent_children.is_empty());
}

#[test]
fn parent_container_change_rebuilds_the_parent_dependency_projection() {
    const CHILD_COUNT: u64 = 64;
    let parent_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.dependencies.container-fallback"));
    tree.insert_root(
        UiTreeNode::new(parent_id, UiNodePath::new("root")).with_container(UiContainerKind::Free),
    );
    for child_index in 0..CHILD_COUNT {
        let child_id = UiNodeId::new(child_index + 2);
        tree.insert_child(
            parent_id,
            UiTreeNode::new(
                child_id,
                UiNodePath::new(format!("root.child-{child_index}")),
            ),
        )
        .expect("insert child");
    }
    let slot_index = UiLayoutSlotIndex::for_tree(&tree);
    tree.clear_pending_mutation_node_ids();
    let initial_evaluations = slot_index.parent_size_dependency_evaluations();

    tree.node_mut(parent_id).expect("parent").container = UiContainerKind::Container;
    slot_index.synchronize_parent_size_dependencies(&tree, &BTreeSet::from([parent_id]));

    assert_eq!(
        slot_index.parent_size_dependency_evaluations() - initial_evaluations,
        CHILD_COUNT as usize
    );
}
