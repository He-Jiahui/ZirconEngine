use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        Anchor, DesiredSize, UiAxis, UiContainerKind, UiFrame, UiMasonryBoxConfig, UiScrollState,
        UiScrollableBoxConfig, UiSize, UiSlot, UiSlotKind, UiVirtualListConfig, UiWrapBoxConfig,
    },
    tree::{UiTree, UiTreeError, UiTreeNode, UiVisibility},
};

use std::{collections::BTreeSet, sync::Arc};

use super::arrange_node;
use crate::ui::layout::pass::{
    axis::resolve_linear_child_main_extents, engine::UiLayoutPassEngineContext,
    slot::UiLayoutSlotIndex,
};

#[test]
fn ten_thousand_sibling_arrangement_reuses_depth_scoped_child_scratch() {
    const CHILD_COUNT: u64 = 10_000;

    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.workspace.scale"));
    tree.insert_root(node(1).with_container(UiContainerKind::SizeBox(Default::default())));
    for child in 0..CHILD_COUNT {
        tree.insert_child(root_id, node(child + 2))
            .expect("insert scale child");
    }

    let slot_index = UiLayoutSlotIndex::default();
    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 1_000.0, 800.0),
        &slot_index,
    )
    .expect("first scale arrangement");
    let first_capacities = arrange_pool_capacities(&slot_index);
    assert_eq!(first_capacities.len(), 2);
    assert!(first_capacities
        .iter()
        .any(|children| *children >= CHILD_COUNT as usize));

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 1_000.0, 800.0),
        &slot_index,
    )
    .expect("stable scale arrangement");

    assert_eq!(arrange_pool_capacities(&slot_index), first_capacities);
}

#[test]
fn taffy_bridge_reuses_tree_and_buffers_across_repeated_arrangement() {
    const CHILD_COUNT: u64 = 256;
    const ACTIVE_CHILD_COUNT: usize = CHILD_COUNT as usize / 2;

    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.workspace"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    for child_id in 2..CHILD_COUNT + 2 {
        let child = if child_id % 2 == 0 {
            node(child_id)
        } else {
            node(child_id).with_visibility(UiVisibility::Collapsed)
        };
        tree.insert_child(root_id, child)
            .expect("insert Taffy child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 4_096.0, 80.0);

    let first_report =
        arrange(&mut tree, root_id, frame, &slot_index).expect("first Taffy arrangement");
    let first_frames = tree
        .node(root_id)
        .expect("Taffy root")
        .children
        .iter()
        .map(|child_id| {
            tree.node(*child_id)
                .expect("Taffy child")
                .layout_cache
                .frame
        })
        .collect::<Vec<_>>();
    assert_eq!(first_report.taffy_tree_build_count, 1);
    assert_eq!(
        first_report.taffy_tree_node_count,
        ACTIVE_CHILD_COUNT as u64 + 1
    );
    assert_eq!(
        slot_index.taffy_parent_product_counts(),
        (1, ACTIVE_CHILD_COUNT + 1)
    );

    let second_report =
        arrange(&mut tree, root_id, frame, &slot_index).expect("reused Taffy arrangement");
    let second_frames = tree
        .node(root_id)
        .expect("Taffy root")
        .children
        .iter()
        .map(|child_id| {
            tree.node(*child_id)
                .expect("Taffy child")
                .layout_cache
                .frame
        })
        .collect::<Vec<_>>();

    assert_eq!(second_frames, first_frames);
    assert_eq!(second_report.taffy_tree_build_count, 0);
    assert_eq!(second_report.taffy_tree_node_count, 0);
    assert_eq!(
        slot_index.taffy_parent_product_counts(),
        (1, ACTIVE_CHILD_COUNT + 1)
    );
}

#[test]
fn taffy_parent_product_patches_one_child_without_recreating_stable_nodes() {
    let root_id = UiNodeId::new(1);
    let changed_child_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.local-style"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    tree.insert_child(root_id, node(2))
        .expect("insert changed Taffy child");
    tree.insert_child(root_id, node(3))
        .expect("insert stable Taffy sibling");
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 200.0, 40.0);

    let first = arrange(&mut tree, root_id, frame, &slot_index).expect("warm Taffy product");
    let stable_sibling_frame = tree
        .node(UiNodeId::new(3))
        .expect("stable sibling")
        .layout_cache
        .frame;
    tree.node_mut(changed_child_id)
        .expect("changed child")
        .layout_cache
        .desired_size = DesiredSize::new(30.0, 10.0);

    let patched = arrange(&mut tree, root_id, frame, &slot_index).expect("patch Taffy product");

    assert_eq!(first.taffy_tree_node_count, 3);
    assert_eq!(patched.taffy_tree_build_count, 0);
    assert_eq!(patched.taffy_tree_node_count, 0);
    assert_eq!(
        tree.node(changed_child_id)
            .expect("changed child")
            .layout_cache
            .frame
            .width,
        30.0
    );
    assert_eq!(
        tree.node(UiNodeId::new(3))
            .expect("stable sibling")
            .layout_cache
            .frame,
        UiFrame::new(
            30.0,
            stable_sibling_frame.y,
            10.0,
            stable_sibling_frame.height
        )
    );
}

#[test]
fn taffy_parent_product_creates_only_an_inserted_child_node() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.insert"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    tree.insert_child(root_id, node(2))
        .expect("insert initial Taffy child");
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 200.0, 40.0);

    let first = arrange(&mut tree, root_id, frame, &slot_index).expect("warm Taffy product");
    tree.insert_child(root_id, node(3))
        .expect("insert incremental Taffy child");
    let patched = arrange(&mut tree, root_id, frame, &slot_index).expect("reconcile Taffy product");

    assert_eq!(first.taffy_tree_node_count, 2);
    assert_eq!(patched.taffy_tree_build_count, 1);
    assert_eq!(patched.taffy_tree_node_count, 1);
    assert_eq!(slot_index.taffy_parent_product_counts(), (1, 3));
}

#[test]
fn taffy_parent_product_root_resize_reuses_all_child_contracts() {
    const CHILD_COUNT: u64 = 256;

    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.root-resize-contract"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    for child_id in 2..CHILD_COUNT + 2 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert retained Taffy child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 2_560.0, 40.0),
        &slot_index,
    )
    .expect("warm retained Taffy product");
    tree.clear_pending_mutation_node_ids();

    let mut engine_context = UiLayoutPassEngineContext::incremental(Default::default());
    engine_context.index_required_children(&tree);
    arrange_node(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 2_600.0, 40.0),
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("resize retained Taffy product");

    let update = slot_index
        .taffy_parent_product_last_update(root_id)
        .expect("retained parent update stats");
    assert!(update.exact_contract_delta);
    assert_eq!(update.child_contract_visit_count, 0);
    assert_eq!(update.child_style_update_count, 0);
    assert!(!update.compute_reused);
    assert_eq!(update.child_layout_read_count, CHILD_COUNT as usize);
    assert_eq!(update.published_child_frame_count, CHILD_COUNT as usize);
}

#[test]
fn taffy_parent_product_patches_only_the_required_direct_child_contract() {
    const CHILD_COUNT: u64 = 256;

    let root_id = UiNodeId::new(1);
    let changed_child_id = UiNodeId::new(129);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.child-contract-delta"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    for child_id in 2..CHILD_COUNT + 2 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert retained Taffy child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 2_560.0, 40.0);
    arrange(&mut tree, root_id, frame, &slot_index).expect("warm retained Taffy product");
    tree.clear_pending_mutation_node_ids();
    tree.node_mut(changed_child_id)
        .expect("changed child")
        .layout_cache
        .desired_size = DesiredSize::new(20.0, 10.0);

    let required = [root_id, changed_child_id].into_iter().collect();
    let sources = [changed_child_id].into_iter().collect();
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(required, sources);
    engine_context.index_required_children(&tree);
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("patch retained child contract");

    let update = slot_index
        .taffy_parent_product_last_update(root_id)
        .expect("retained parent update stats");
    assert!(update.exact_contract_delta);
    assert_eq!(update.child_contract_visit_count, 1);
    assert_eq!(update.child_style_update_count, 1);
    assert!(!update.compute_reused);
    assert_eq!(update.child_layout_read_count, CHILD_COUNT as usize);
    assert_eq!(update.published_child_frame_count, CHILD_COUNT as usize);
    assert_eq!(
        tree.node(changed_child_id)
            .expect("changed child")
            .layout_cache
            .frame
            .width,
        20.0
    );
}

#[test]
fn taffy_exact_unchanged_contract_reuses_solve_and_publishes_only_required_child() {
    const CHILD_COUNT: u64 = 256;

    let root_id = UiNodeId::new(1);
    let changed_child_id = UiNodeId::new(129);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.solve-reuse"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    for child_id in 2..CHILD_COUNT + 2 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert retained Taffy child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 2_560.0, 40.0);
    arrange(&mut tree, root_id, frame, &slot_index).expect("warm retained Taffy product");
    tree.clear_pending_mutation_node_ids();
    tree.node_mut(changed_child_id)
        .expect("changed child")
        .dirty
        .input = true;

    let required = [root_id, changed_child_id].into_iter().collect();
    let sources = [changed_child_id].into_iter().collect();
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(required, sources);
    engine_context.index_required_children(&tree);
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("reuse retained solve");

    let update = slot_index
        .taffy_parent_product_last_update(root_id)
        .expect("retained parent update stats");
    assert!(update.exact_contract_delta);
    assert!(update.compute_reused);
    assert_eq!(update.child_contract_visit_count, 1);
    assert_eq!(update.child_style_update_count, 0);
    assert_eq!(update.child_layout_read_count, 0);
    assert_eq!(update.published_child_frame_count, 1);
    let (_, _, _, arrangement_probe_count) = engine_context.finish_incremental();
    assert_eq!(arrangement_probe_count, 2);
}

#[test]
fn taffy_parent_translation_reuses_solve_but_publishes_all_children() {
    const CHILD_COUNT: u64 = 256;

    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.translation"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    for child_id in 2..CHILD_COUNT + 2 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert retained Taffy child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 2_560.0, 40.0),
        &slot_index,
    )
    .expect("warm retained Taffy product");
    tree.clear_pending_mutation_node_ids();

    let mut engine_context = UiLayoutPassEngineContext::incremental(Default::default());
    engine_context.index_required_children(&tree);
    arrange_node(
        &mut tree,
        root_id,
        UiFrame::new(10.0, 20.0, 2_560.0, 40.0),
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("translate retained product");

    let update = slot_index
        .taffy_parent_product_last_update(root_id)
        .expect("retained parent update stats");
    assert!(update.exact_contract_delta);
    assert!(update.compute_reused);
    assert_eq!(update.child_contract_visit_count, 0);
    assert_eq!(update.child_layout_read_count, 0);
    assert_eq!(update.published_child_frame_count, CHILD_COUNT as usize);
    assert_eq!(
        tree.node(UiNodeId::new(2))
            .expect("translated first child")
            .layout_cache
            .frame
            .x,
        10.0
    );
}

#[test]
fn taffy_exact_contract_receipt_falls_back_when_active_membership_changes() {
    let root_id = UiNodeId::new(1);
    let restored_child_id = UiNodeId::new(3);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.membership-fallback"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    tree.insert_child(root_id, node(2))
        .expect("insert active Taffy child");
    tree.insert_child(root_id, node(3).with_visibility(UiVisibility::Collapsed))
        .expect("insert collapsed Taffy child");
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 200.0, 40.0);
    arrange(&mut tree, root_id, frame, &slot_index).expect("warm retained Taffy product");
    tree.clear_pending_mutation_node_ids();
    tree.node_mut(restored_child_id)
        .expect("restored child")
        .visibility = UiVisibility::Visible;

    let required = [root_id, restored_child_id].into_iter().collect();
    let sources = [restored_child_id].into_iter().collect();
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(required, sources);
    engine_context.index_required_children(&tree);
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("reconcile restored child membership");

    let update = slot_index
        .taffy_parent_product_last_update(root_id)
        .expect("retained parent update stats");
    assert!(!update.exact_contract_delta);
    assert_eq!(update.child_contract_visit_count, 2);
    assert_eq!(slot_index.taffy_parent_product_counts(), (1, 3));
}

#[test]
fn taffy_exact_contract_receipt_falls_back_when_child_order_revision_changes() {
    let root_id = UiNodeId::new(1);
    let first_child_id = UiNodeId::new(2);
    let second_child_id = UiNodeId::new(3);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.taffy.order-fallback"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    tree.insert_child(root_id, node(2))
        .expect("insert first Taffy child");
    tree.insert_child(root_id, node(3))
        .expect("insert second Taffy child");
    let slot_index = UiLayoutSlotIndex::default();
    let frame = UiFrame::new(0.0, 0.0, 200.0, 40.0);
    arrange(&mut tree, root_id, frame, &slot_index).expect("warm retained Taffy product");
    tree.clear_pending_mutation_node_ids();
    tree.node_mut(root_id)
        .expect("Taffy parent")
        .children
        .swap(0, 1);
    slot_index.synchronize_ordered_children(&tree, &BTreeSet::from([root_id]));

    let required = [root_id, first_child_id].into_iter().collect();
    let sources = [first_child_id].into_iter().collect();
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(required, sources);
    engine_context.index_required_children(&tree);
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("reconcile reordered retained children");

    let update = slot_index
        .taffy_parent_product_last_update(root_id)
        .expect("retained parent update stats");
    assert!(!update.exact_contract_delta);
    assert_eq!(update.child_contract_visit_count, 2);
    assert_eq!(
        tree.node(second_child_id)
            .expect("reordered first child")
            .layout_cache
            .frame
            .x,
        0.0
    );
}

#[test]
fn equal_slot_order_reuses_the_cached_generation() {
    let root_id = UiNodeId::new(1);
    let container = UiContainerKind::HorizontalBox(Default::default());
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.stable.order"));
    tree.insert_root(node(1).with_container(container));
    for child_id in 2..=4 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert ordered child");
    }
    tree.replace_layout_slots(vec![
        UiSlot::new(root_id, UiNodeId::new(2), UiSlotKind::Linear).with_order(1),
        UiSlot::new(root_id, UiNodeId::new(3), UiSlotKind::Linear).with_order(1),
        UiSlot::new(root_id, UiNodeId::new(4), UiSlotKind::Linear).with_order(0),
    ]);
    let slot_index = UiLayoutSlotIndex::default();

    let ordered = slot_index.ordered_children_for_container(&tree, root_id, container);
    let stable = slot_index.ordered_children_for_container(&tree, root_id, container);

    assert_eq!(
        ordered.as_ref(),
        &[UiNodeId::new(4), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert!(Arc::ptr_eq(&ordered, &stable));
}

#[test]
fn free_arrangement_preserves_slot_order() {
    let root_id = UiNodeId::new(1);
    let container = UiContainerKind::Free;
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.free.tree.order"));
    tree.insert_root(node(1).with_container(container));
    for child_id in 2..=4 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert free child");
    }
    tree.replace_layout_slots(vec![
        UiSlot::new(root_id, UiNodeId::new(2), UiSlotKind::Free).with_order(2),
        UiSlot::new(root_id, UiNodeId::new(3), UiSlotKind::Free).with_order(1),
        UiSlot::new(root_id, UiNodeId::new(4), UiSlotKind::Free).with_order(0),
    ]);
    let slot_index = UiLayoutSlotIndex::default();
    let ordered = slot_index.ordered_children_for_container(&tree, root_id, container);

    assert_eq!(
        ordered.as_ref(),
        &[UiNodeId::new(4), UiNodeId::new(3), UiNodeId::new(2)]
    );
}

#[test]
fn scrollable_arrangement_preserves_tree_order_despite_slot_order_metadata() {
    let root_id = UiNodeId::new(1);
    let container = UiContainerKind::ScrollableBox(UiScrollableBoxConfig::default());
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.scrollable.tree.order"));
    tree.insert_root(node(1).with_container(container));
    for child_id in 2..=4 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert scrollable child");
    }
    tree.replace_layout_slots(vec![
        UiSlot::new(root_id, UiNodeId::new(2), UiSlotKind::Scrollable).with_order(2),
        UiSlot::new(root_id, UiNodeId::new(3), UiSlotKind::Scrollable).with_order(1),
        UiSlot::new(root_id, UiNodeId::new(4), UiSlotKind::Scrollable).with_order(0),
    ]);
    let slot_index = UiLayoutSlotIndex::default();
    let ordered = slot_index.ordered_children_for_container(&tree, root_id, container);

    assert_eq!(
        ordered.as_ref(),
        &[UiNodeId::new(2), UiNodeId::new(3), UiNodeId::new(4)]
    );
}

#[test]
fn materialized_virtual_list_places_physical_slot_at_logical_offset() {
    let (mut tree, slot_index, children, frame) =
        materialized_virtual_list_fixture(100_000, 50_000, 4);

    arrange(&mut tree, UiNodeId::new(1), frame, &slot_index)
        .expect("arrange materialized virtual list");

    assert_eq!(
        tree.node(children[0])
            .expect("first slot")
            .layout_cache
            .frame,
        UiFrame::new(10.0, 20.0, 10.0, 24.0)
    );
    assert_eq!(
        tree.node(children[1])
            .expect("second slot")
            .layout_cache
            .frame,
        UiFrame::new(10.0, 46.0, 10.0, 24.0)
    );
}

#[test]
fn materialized_virtual_list_keeps_logical_content_extent() {
    let (mut tree, slot_index, _, frame) = materialized_virtual_list_fixture(100_000, 50_000, 4);

    arrange(&mut tree, UiNodeId::new(1), frame, &slot_index)
        .expect("arrange materialized virtual-list content extent");

    let owner = tree.node(UiNodeId::new(1)).expect("virtual-list owner");
    assert_eq!(owner.layout_cache.content_size.height, 2_599_998.0);
    assert_eq!(
        owner
            .scroll_state
            .expect("resolved scroll state")
            .content_extent,
        2_599_998.0
    );
}

#[test]
fn materialized_virtual_list_arranges_backfilled_slot_outside_visible_window() {
    let (mut tree, slot_index, children, frame) = materialized_virtual_list_fixture(100, 0, 5);

    arrange(&mut tree, UiNodeId::new(1), frame, &slot_index)
        .expect("arrange backfilled materialized slot");

    assert_eq!(
        tree.node(UiNodeId::new(1))
            .expect("virtual-list owner")
            .layout_cache
            .virtual_window,
        Some(zircon_runtime_interface::ui::layout::UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(
        tree.node(children[4])
            .expect("backfilled slot")
            .layout_cache
            .frame,
        UiFrame::new(10.0, 124.0, 10.0, 24.0)
    );
}

#[test]
fn materialized_virtual_list_arrangement_visits_only_physical_slots() {
    assert_eq!(materialized_virtual_list_probe_count(100), 5);
    assert_eq!(materialized_virtual_list_probe_count(100_000), 5);
}

#[test]
fn wrap_content_size_uses_the_same_cached_order_as_arrangement() {
    let root_id = UiNodeId::new(1);
    let config = UiWrapBoxConfig::default();
    let container = UiContainerKind::WrapBox(config);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.wrap.content.order"));
    tree.insert_root(node(1).with_container(container));
    for (child_id, desired) in [
        (2, DesiredSize::new(6.0, 100.0)),
        (3, DesiredSize::new(4.0, 90.0)),
        (4, DesiredSize::new(6.0, 1.0)),
        (5, DesiredSize::new(4.0, 1.0)),
    ] {
        tree.insert_child(root_id, node_with_desired(child_id, desired))
            .expect("insert wrap child");
    }
    tree.replace_layout_slots(vec![
        UiSlot::new(root_id, UiNodeId::new(2), UiSlotKind::Flow).with_order(0),
        UiSlot::new(root_id, UiNodeId::new(4), UiSlotKind::Flow).with_order(1),
        UiSlot::new(root_id, UiNodeId::new(3), UiSlotKind::Flow).with_order(2),
        UiSlot::new(root_id, UiNodeId::new(5), UiSlotKind::Flow).with_order(3),
    ]);
    tree.node_mut(UiNodeId::new(2))
        .expect("first wrap child")
        .anchor = Anchor::new(0.25, 0.0);
    let slot_index = UiLayoutSlotIndex::default();

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 10.0, 300.0),
        &slot_index,
    )
    .expect("arrange wrap content");

    assert_eq!(
        tree.node(root_id)
            .expect("wrap root")
            .layout_cache
            .content_size,
        UiSize::new(10.0, 191.0)
    );
    let first_capacity = container_arrange_capacities(&slot_index);
    assert!(first_capacity
        .iter()
        .any(|(row, desired, _, _)| *row > 0 && *desired >= 4));

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 10.0, 300.0),
        &slot_index,
    )
    .expect("reuse wrap content buffers");
    assert_eq!(container_arrange_capacities(&slot_index), first_capacity);
}

#[test]
fn scrollable_arrangement_streams_positions_across_collapsed_children() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.scrollable.positions"));
    tree.insert_root(node(1).with_container(UiContainerKind::ScrollableBox(
        UiScrollableBoxConfig {
            axis: UiAxis::Vertical,
            gap: 2.0,
            ..UiScrollableBoxConfig::default()
        },
    )));
    tree.insert_child(root_id, node(2))
        .expect("insert first child");
    tree.insert_child(root_id, node(3).with_visibility(UiVisibility::Collapsed))
        .expect("insert collapsed child");
    tree.insert_child(root_id, node(4))
        .expect("insert final child");
    let slot_index = UiLayoutSlotIndex::default();

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 80.0),
        &slot_index,
    )
    .expect("arrange streamed positions");

    assert_eq!(
        tree.node(UiNodeId::new(2))
            .expect("first")
            .layout_cache
            .frame
            .y,
        0.0
    );
    assert_eq!(
        tree.node(UiNodeId::new(3))
            .expect("collapsed")
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        tree.node(UiNodeId::new(4))
            .expect("final")
            .layout_cache
            .frame
            .y,
        12.0
    );
}

#[test]
fn scrollable_arrangement_validates_all_direct_children_before_mutating_subtrees() {
    let root_id = UiNodeId::new(1);
    let first_id = UiNodeId::new(2);
    let missing_direct_id = UiNodeId::new(3);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.scrollable.validation.order"));
    tree.insert_root(node(1).with_container(UiContainerKind::ScrollableBox(
        UiScrollableBoxConfig::default(),
    )));
    tree.insert_child(root_id, node(2))
        .expect("insert first child");
    tree.node_mut(first_id)
        .expect("first child")
        .children
        .push(UiNodeId::new(99));
    tree.node_mut(first_id)
        .expect("first child")
        .layout_cache
        .frame = UiFrame::new(7.0, 8.0, 9.0, 10.0);
    tree.node_mut(root_id)
        .expect("root")
        .children
        .push(missing_direct_id);
    let slot_index = UiLayoutSlotIndex::default();

    let error = arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 80.0),
        &slot_index,
    )
    .expect_err("later direct child should fail validation");

    assert!(matches!(
        error,
        UiTreeError::MissingNode(node_id) if node_id == missing_direct_id
    ));
    assert_eq!(
        tree.node(first_id).expect("first child").layout_cache.frame,
        UiFrame::new(7.0, 8.0, 9.0, 10.0)
    );
}

#[test]
fn masonry_arrangement_reuses_column_buffers() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.masonry.workspace"));
    tree.insert_root(
        node(1).with_container(UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: 4,
            gap: 2.0,
            sequential: false,
        })),
    );
    for child_id in 2..=33 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert masonry child");
    }
    let slot_index = UiLayoutSlotIndex::default();

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 200.0),
        &slot_index,
    )
    .expect("first masonry arrangement");
    let first_capacity = container_arrange_capacities(&slot_index);
    assert!(first_capacity
        .iter()
        .any(|(_, _, heights, counts)| *heights >= 4 && *counts >= 4));

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 200.0),
        &slot_index,
    )
    .expect("reuse masonry buffers");
    assert_eq!(container_arrange_capacities(&slot_index), first_capacity);
}

#[test]
fn hidden_subtree_arrangement_reuses_one_iterative_stack() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.hidden.workspace"));
    tree.insert_root(node(1).with_container(UiContainerKind::Space));
    let mut parent_id = root_id;
    for child_id in 2..=128 {
        tree.insert_child(parent_id, node(child_id))
            .expect("insert hidden chain child");
        parent_id = UiNodeId::new(child_id);
    }
    let slot_index = UiLayoutSlotIndex::default();

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 80.0),
        &slot_index,
    )
    .expect("first hidden subtree arrangement");
    let first_capacity = hidden_subtree_stack_capacity(&slot_index);
    assert!(first_capacity > 0);

    arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 80.0),
        &slot_index,
    )
    .expect("stable hidden subtree arrangement");
    assert_eq!(hidden_subtree_stack_capacity(&slot_index), first_capacity);
}

#[test]
fn incremental_hidden_subtree_rejects_redundant_zero_geometry_walks() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let frame = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.hidden.incremental"));
    tree.insert_root(node(1).with_container(UiContainerKind::Space));
    tree.insert_child(root_id, node(2)).expect("insert child");
    let slot_index = UiLayoutSlotIndex::default();
    arrange(&mut tree, root_id, frame, &slot_index).expect("seed hidden geometry");

    let mut engine_context =
        UiLayoutPassEngineContext::incremental([root_id].into_iter().collect());
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("incremental hidden arrangement");
    let (_, visited, changed, _) = engine_context.finish_incremental();

    assert_eq!(visited, [root_id].into_iter().collect());
    assert!(!visited.contains(&child_id));
    assert!(changed.is_empty());
}

#[test]
fn incremental_free_parent_arranges_only_required_direct_children() {
    let root_id = UiNodeId::new(1);
    let changed_child_id = UiNodeId::new(2);
    let frame = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.sparse.free"));
    tree.insert_root(node(1).with_container(UiContainerKind::Free));
    for child_id in 2..=101 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert sparse child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    arrange(&mut tree, root_id, frame, &slot_index).expect("seed sparse geometry");

    let required = [root_id, changed_child_id].into_iter().collect();
    let mut engine_context =
        UiLayoutPassEngineContext::incremental_with_sources(required, Default::default());
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("sparse incremental arrangement");

    let (_, visited, changed, probe_count) = engine_context.finish_incremental();
    assert_eq!(visited, [root_id, changed_child_id].into_iter().collect());
    assert!(changed.is_empty());
    assert_eq!(probe_count, 2);
}

#[test]
fn incremental_free_parent_source_keeps_full_child_arrangement() {
    let root_id = UiNodeId::new(1);
    let changed_child_id = UiNodeId::new(2);
    let frame = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.full.source"));
    tree.insert_root(node(1).with_container(UiContainerKind::Free));
    for child_id in 2..=101 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert source child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    arrange(&mut tree, root_id, frame, &slot_index).expect("seed source geometry");

    let required = [root_id, changed_child_id].into_iter().collect();
    let sources = [root_id].into_iter().collect();
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(required, sources);
    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("full source arrangement");

    let (_, visited, changed, probe_count) = engine_context.finish_incremental();
    assert_eq!(visited, [root_id, changed_child_id].into_iter().collect());
    assert!(changed.is_empty());
    assert_eq!(probe_count, 101);
}

#[test]
fn linear_arrangement_solver_reuses_constraints_and_active_indices() {
    const CHILD_COUNT: u64 = 256;

    let root_id = UiNodeId::new(1);
    let container = UiContainerKind::HorizontalBox(Default::default());
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.linear.workspace"));
    tree.insert_root(node(1).with_container(container));
    for child_id in 2..CHILD_COUNT + 2 {
        tree.insert_child(root_id, node(child_id))
            .expect("insert linear child");
    }
    let slot_index = UiLayoutSlotIndex::default();
    let mut scratch = slot_index.take_arrange_child_scratch();
    let ordered = slot_index.ordered_children_for_container(&tree, root_id, container);
    scratch.children.extend_from_slice(&ordered);

    for available in [4_000.0, 1_000.0] {
        resolve_linear_child_main_extents(
            &tree,
            root_id,
            &scratch.children,
            UiAxis::Horizontal,
            available,
            0.0,
            &slot_index,
            &mut scratch.linear,
        )
        .expect("resolve linear extents");
    }
    let first_capacity = linear_scratch_capacity(&scratch);

    for available in [4_000.0, 1_000.0] {
        resolve_linear_child_main_extents(
            &tree,
            root_id,
            &scratch.children,
            UiAxis::Horizontal,
            available,
            0.0,
            &slot_index,
            &mut scratch.linear,
        )
        .expect("reuse linear extents");
    }

    assert_eq!(linear_scratch_capacity(&scratch), first_capacity);
    assert!(scratch.linear.constraints.capacity() >= CHILD_COUNT as usize);
    assert!(scratch.linear.resolved.capacity() >= CHILD_COUNT as usize);
}

#[test]
fn failed_arrangement_recycles_child_scratch() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.workspace.error"));
    tree.insert_root(node(1).with_container(UiContainerKind::Free));
    tree.node_mut(root_id)
        .expect("root")
        .children
        .push(UiNodeId::new(2));
    let slot_index = UiLayoutSlotIndex::default();

    assert!(arrange(
        &mut tree,
        root_id,
        UiFrame::new(0.0, 0.0, 100.0, 80.0),
        &slot_index,
    )
    .is_err());

    assert_eq!(arrange_pool_capacities(&slot_index).len(), 1);
}

fn arrange(
    tree: &mut UiTree,
    root_id: UiNodeId,
    frame: UiFrame,
    slot_index: &UiLayoutSlotIndex,
) -> Result<
    zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    zircon_runtime_interface::ui::tree::UiTreeError,
> {
    let mut engine_context = UiLayoutPassEngineContext::default();
    arrange_node(tree, root_id, frame, None, slot_index, &mut engine_context)?;
    Ok(engine_context.finish())
}

fn materialized_virtual_list_fixture(
    logical_count: usize,
    first_logical_index: usize,
    physical_slot_count: usize,
) -> (UiTree, UiLayoutSlotIndex, Vec<UiNodeId>, UiFrame) {
    let root_id = UiNodeId::new(1);
    let item_extent = 24.0;
    let gap = 2.0;
    let step_extent = item_extent + gap;
    let offset = first_logical_index as f32 * step_extent;
    let config = UiScrollableBoxConfig {
        axis: UiAxis::Vertical,
        gap,
        virtualization: Some(UiVirtualListConfig {
            item_extent,
            overscan: 0,
        }),
        ..UiScrollableBoxConfig::default()
    };
    let mut tree = UiTree::new(UiTreeId::new("layout.arrange.materialized.virtual-list"));
    tree.insert_root(
        node(1)
            .with_container(UiContainerKind::ScrollableBox(config))
            .with_scroll_state(UiScrollState {
                offset,
                viewport_extent: 80.0,
                content_extent: 0.0,
            }),
    );
    let mut children = Vec::with_capacity(physical_slot_count);
    for slot_index in 0..physical_slot_count {
        let child_id = UiNodeId::new(2 + slot_index as u64);
        tree.insert_child(
            root_id,
            node_with_desired(2 + slot_index as u64, DesiredSize::new(10.0, item_extent)),
        )
        .expect("insert materialized row slot");
        children.push(child_id);
    }
    let slot_index = UiLayoutSlotIndex::default();
    slot_index.replace_materialized_virtual_list(
        root_id,
        logical_count,
        children
            .iter()
            .copied()
            .enumerate()
            .map(|(slot, child_id)| (child_id, first_logical_index + slot)),
    );
    (
        tree,
        slot_index,
        children,
        UiFrame::new(10.0, 20.0, 100.0, 80.0),
    )
}

fn materialized_virtual_list_probe_count(logical_count: usize) -> usize {
    let first_logical_index = logical_count / 2;
    let (mut tree, slot_index, children, frame) =
        materialized_virtual_list_fixture(logical_count, first_logical_index, 4);
    let root_id = UiNodeId::new(1);
    let required = std::iter::once(root_id)
        .chain(children.iter().copied())
        .collect();
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(
        required,
        [root_id].into_iter().collect(),
    );
    engine_context.index_required_children(&tree);

    arrange_node(
        &mut tree,
        root_id,
        frame,
        None,
        &slot_index,
        &mut engine_context,
    )
    .expect("arrange bounded materialized virtual list");

    engine_context.finish_incremental().3
}

fn arrange_pool_capacities(slot_index: &UiLayoutSlotIndex) -> Vec<usize> {
    slot_index.with_measure_workspace(|workspace| {
        workspace
            .arrange_child_pool
            .iter()
            .map(|scratch| scratch.children.capacity())
            .collect()
    })
}

fn hidden_subtree_stack_capacity(slot_index: &UiLayoutSlotIndex) -> usize {
    slot_index.with_measure_workspace(|workspace| workspace.hidden_subtree_stack.capacity())
}

fn linear_scratch_capacity(
    scratch: &crate::ui::layout::pass::workspace::UiArrangeChildScratch,
) -> (usize, usize, usize, usize) {
    (
        scratch.linear.constraints.capacity(),
        scratch.linear.resolved.capacity(),
        scratch.linear.priorities.capacity(),
        scratch.linear.active_indices.capacity(),
    )
}

fn container_arrange_capacities(
    slot_index: &UiLayoutSlotIndex,
) -> Vec<(usize, usize, usize, usize)> {
    slot_index.with_measure_workspace(|workspace| {
        workspace
            .arrange_child_pool
            .iter()
            .map(|scratch| {
                (
                    scratch.wrap_row_items.capacity(),
                    scratch.wrap_content_desired.capacity(),
                    scratch.masonry.column_heights.capacity(),
                    scratch.masonry.column_counts.capacity(),
                )
            })
            .collect()
    })
}

fn node(id: u64) -> UiTreeNode {
    node_with_desired(id, DesiredSize::new(10.0, 10.0))
}

fn node_with_desired(id: u64, desired_size: DesiredSize) -> UiTreeNode {
    let mut node = UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("node.{id}")));
    node.layout_cache.desired_size = desired_size;
    node
}
