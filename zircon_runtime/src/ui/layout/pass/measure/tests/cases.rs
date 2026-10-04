use std::{collections::BTreeSet, sync::Arc};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, DesiredSize, StretchMode, UiContainerKind, UiFrame,
        UiGridBoxConfig, UiMasonryBoxConfig, UiSize, UiSlot, UiSlotKind,
    },
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode, UiVisibility},
};

use super::{exact_fixed_width, measure_node, measure_node_incremental};
use crate::ui::layout::pass::slot::UiLayoutSlotIndex;
use crate::ui::text::UiTextMeasureCache;

#[test]
fn exact_fixed_width_requires_an_equal_minimum_and_maximum() {
    let exact = BoxConstraints {
        width: AxisConstraint {
            min: 320.0,
            max: 320.0,
            preferred: 320.0,
            stretch_mode: StretchMode::Fixed,
            ..AxisConstraint::default()
        },
        ..BoxConstraints::default()
    };
    let flexible = BoxConstraints {
        width: AxisConstraint {
            min: 0.0,
            max: -1.0,
            preferred: 320.0,
            stretch_mode: StretchMode::Fixed,
            ..AxisConstraint::default()
        },
        ..BoxConstraints::default()
    };

    assert_eq!(exact_fixed_width(exact), Some(320.0));
    assert_eq!(exact_fixed_width(flexible), None);
}

#[test]
fn post_order_measurement_collapses_the_entire_hidden_subtree() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.collapsed"));
    tree.insert_root(
        node(1)
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_visibility(UiVisibility::Collapsed),
    );
    tree.insert_child(root_id, node(2)).expect("insert child");
    tree.node_mut(root_id)
        .expect("root")
        .layout_cache
        .desired_size = DesiredSize::new(200.0, 100.0);
    tree.node_mut(child_id)
        .expect("child")
        .layout_cache
        .desired_size = DesiredSize::new(80.0, 40.0);

    let slot_index = UiLayoutSlotIndex::default();
    let mut text_cache = UiTextMeasureCache::default();
    let desired =
        measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("measure subtree");

    assert_eq!(desired, DesiredSize::default());
    for node_id in [root_id, child_id] {
        let cache = tree.node(node_id).expect("measured node").layout_cache;
        assert_eq!(cache.desired_size, DesiredSize::default());
        assert_eq!(cache.content_size, UiSize::default());
        assert_eq!(cache.virtual_window, None);
    }
}

#[test]
fn incremental_measurement_rebuilds_a_subtree_restored_from_collapsed() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.restore.collapsed"));
    tree.insert_root(
        node(1)
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_visibility(UiVisibility::Collapsed),
    );
    tree.insert_child(root_id, fixed_node(2, 40.0, 20.0))
        .expect("insert child");
    let slot_index = UiLayoutSlotIndex::default();
    let mut text_cache = UiTextMeasureCache::default();
    measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("collapse measure");
    assert_eq!(
        tree.node(child_id).unwrap().layout_cache.desired_size,
        DesiredSize::default()
    );

    tree.node_mut(root_id).unwrap().visibility = UiVisibility::Visible;
    let required = [root_id].into_iter().collect();
    let mut visited = Default::default();
    measure_node_incremental(
        &mut tree,
        root_id,
        &mut text_cache,
        &slot_index,
        &required,
        &mut visited,
    )
    .expect("restored subtree measure");

    assert_eq!(
        tree.node(child_id).unwrap().layout_cache.desired_size,
        DesiredSize::new(40.0, 20.0)
    );
    assert_eq!(visited, [root_id, child_id].into_iter().collect());
}

#[test]
fn incremental_measurement_reuses_valid_zero_frame_parent_without_forcing_clean_descendants() {
    let root_id = UiNodeId::new(1);
    let changed_branch_id = UiNodeId::new(2);
    let changed_leaf_id = UiNodeId::new(3);
    let clean_branch_id = UiNodeId::new(4);
    let clean_leaf_id = UiNodeId::new(5);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.zero-frame.validity"));
    tree.insert_root(node(1).with_container(UiContainerKind::Free));
    tree.insert_child(root_id, node(2).with_container(UiContainerKind::Free))
        .expect("insert changed branch");
    tree.insert_child(root_id, node(4).with_container(UiContainerKind::Free))
        .expect("insert clean branch");
    tree.insert_child(changed_branch_id, node(3))
        .expect("insert changed leaf");
    tree.insert_child(clean_branch_id, node(5))
        .expect("insert clean leaf");

    for node_id in [
        root_id,
        changed_branch_id,
        changed_leaf_id,
        clean_branch_id,
        clean_leaf_id,
    ] {
        let node = tree.node_mut(node_id).expect("validity node");
        node.layout_cache.frame = UiFrame::default();
        node.layout_cache.complete_measure();
        node.dirty = Default::default();
    }

    let required = [root_id, changed_branch_id, changed_leaf_id]
        .into_iter()
        .collect();
    let slot_index = UiLayoutSlotIndex::default();
    let mut visited = BTreeSet::new();
    let mut text_cache = UiTextMeasureCache::default();
    let (_, probe_count) = measure_node_incremental(
        &mut tree,
        root_id,
        &mut text_cache,
        &slot_index,
        &required,
        &mut visited,
    )
    .expect("valid zero-frame incremental measure");

    assert_eq!(
        visited,
        [root_id, changed_branch_id, changed_leaf_id]
            .into_iter()
            .collect()
    );
    assert!(!visited.contains(&clean_branch_id));
    assert!(!visited.contains(&clean_leaf_id));
    assert_eq!(probe_count, 4);
}

#[test]
fn invalid_parent_measurement_does_not_force_valid_sibling_subtrees() {
    const CLEAN_BRANCH_DEPTH: u64 = 256;

    let root_id = UiNodeId::new(1);
    let clean_branch_id = UiNodeId::new(2);
    let new_child_id = UiNodeId::new(10_000);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.invalid-parent.locality"));
    tree.insert_root(node(1).with_container(UiContainerKind::Free));
    tree.insert_child(root_id, node(2).with_container(UiContainerKind::Free))
        .expect("insert clean branch");
    let mut parent_id = clean_branch_id;
    for depth in 0..CLEAN_BRANCH_DEPTH {
        let child_id = UiNodeId::new(depth + 3);
        tree.insert_child(
            parent_id,
            node(depth + 3).with_container(UiContainerKind::Free),
        )
        .expect("insert clean descendant");
        parent_id = child_id;
    }

    let slot_index = UiLayoutSlotIndex::default();
    let mut text_cache = UiTextMeasureCache::default();
    measure_node(&mut tree, root_id, &mut text_cache, &slot_index)
        .expect("warm clean subtree measurement");
    assert!(
        tree.node(clean_branch_id)
            .expect("clean branch")
            .layout_cache
            .measure_valid
    );

    tree.insert_child(root_id, fixed_node(10_000, 40.0, 20.0))
        .expect("insert changed child");
    let required = [root_id, new_child_id].into_iter().collect();
    let mut visited = BTreeSet::new();
    let (_, probe_count) = measure_node_incremental(
        &mut tree,
        root_id,
        &mut text_cache,
        &slot_index,
        &required,
        &mut visited,
    )
    .expect("incremental parent measurement");

    assert_eq!(visited, [root_id, new_child_id].into_iter().collect());
    assert_eq!(probe_count, 3);
    assert!(
        tree.node(clean_branch_id)
            .expect("clean branch")
            .layout_cache
            .measure_valid
    );
}

#[test]
fn ten_thousand_sibling_measurement_reuses_the_retained_workspace_capacity() {
    const CHILD_COUNT: u64 = 10_000;

    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.workspace.scale"));
    tree.insert_root(node(1).with_container(UiContainerKind::HorizontalBox(Default::default())));
    for child in 0..CHILD_COUNT {
        tree.insert_child(root_id, node(child + 2))
            .expect("insert scale child");
    }

    let slot_index = UiLayoutSlotIndex::default();
    let mut text_cache = UiTextMeasureCache::default();
    measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("first scale measure");
    let first_capacity = workspace_capacity(&slot_index);
    assert!(first_capacity.0 >= CHILD_COUNT as usize + 1);
    assert!(first_capacity.1 >= CHILD_COUNT as usize);

    measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("stable scale measure");

    assert_eq!(workspace_capacity(&slot_index), first_capacity);
}

#[test]
fn grid_after_masonry_reuses_extents_and_preserves_hidden_vs_collapsed_measurement() {
    let root_id = UiNodeId::new(1);
    let hidden_id = UiNodeId::new(3);
    let collapsed_id = UiNodeId::new(4);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.container.scratch"));
    tree.insert_root(
        node(1).with_container(UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: 8,
            gap: 0.0,
            sequential: true,
        })),
    );
    for child in 0..16_u64 {
        let mut child_node = fixed_node(child + 2, 10.0, 20.0);
        if child_node.node_id == hidden_id {
            child_node = child_node.with_visibility(UiVisibility::Hidden);
        } else if child_node.node_id == collapsed_id {
            child_node = child_node.with_visibility(UiVisibility::Collapsed);
        }
        tree.insert_child(root_id, child_node)
            .expect("insert mixed visibility child");
    }

    let slot_index = UiLayoutSlotIndex::default();
    let mut text_cache = UiTextMeasureCache::default();
    let masonry =
        measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("measure masonry");
    let masonry_capacity = container_workspace_capacity(&slot_index);

    assert_eq!(masonry, DesiredSize::new(80.0, 40.0));
    assert_eq!(
        tree.node(hidden_id)
            .expect("hidden child")
            .layout_cache
            .desired_size,
        DesiredSize::new(10.0, 20.0)
    );
    assert_eq!(
        tree.node(collapsed_id)
            .expect("collapsed child")
            .layout_cache
            .desired_size,
        DesiredSize::default()
    );

    tree.node_mut(root_id).expect("root").container = UiContainerKind::GridBox(UiGridBoxConfig {
        columns: 3,
        rows: 1,
        column_gap: 0.0,
        row_gap: 0.0,
    });
    let grid =
        measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("measure grid");

    assert_eq!(grid, DesiredSize::new(30.0, 100.0));
    assert_eq!(container_workspace_capacity(&slot_index), masonry_capacity);
}

#[test]
fn equal_slot_order_preserves_original_child_order_in_cached_generation() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.stable.order"));
    let container = UiContainerKind::HorizontalBox(Default::default());
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
fn post_order_measurement_calls_the_text_cache_once_per_leaf_and_reuses_it() {
    let root_id = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("layout.measure.text.cache"));
    tree.insert_root(node(1).with_container(UiContainerKind::VerticalBox(Default::default())));
    tree.insert_child(
        root_id,
        node(2).with_template_metadata(UiTemplateNodeMetadata {
            component: "Text".to_string(),
            attributes: toml::from_str("text = \"post order text\"").expect("text metadata"),
            ..UiTemplateNodeMetadata::default()
        }),
    )
    .expect("insert text child");
    let slot_index = UiLayoutSlotIndex::default();
    let mut text_cache = UiTextMeasureCache::default();
    text_cache.begin_frame();

    let first =
        measure_node(&mut tree, root_id, &mut text_cache, &slot_index).expect("first text measure");
    assert_eq!(text_cache.frame_measure_report().miss_count, 1);
    assert_eq!(text_cache.frame_measure_dedup_report().miss_count, 1);

    let second = measure_node(&mut tree, root_id, &mut text_cache, &slot_index)
        .expect("stable text measure");

    assert_eq!(second, first);
    assert_eq!(text_cache.frame_measure_report().miss_count, 1);
    assert_eq!(text_cache.frame_measure_dedup_report().hit_count, 1);
}

fn node(id: u64) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("node.{id}")))
}

fn fixed_node(id: u64, width: f32, height: f32) -> UiTreeNode {
    node(id).with_constraints(BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    })
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        stretch_mode: StretchMode::Fixed,
        ..AxisConstraint::default()
    }
}

fn workspace_capacity(slot_index: &UiLayoutSlotIndex) -> (usize, usize) {
    slot_index.with_measure_workspace(|workspace| {
        (
            workspace.post_order.capacity(),
            workspace.child_desired.capacity(),
        )
    })
}

fn container_workspace_capacity(slot_index: &UiLayoutSlotIndex) -> (usize, usize, usize) {
    slot_index.with_measure_workspace(|workspace| {
        (
            workspace.container_scratch.primary_extents.capacity(),
            workspace.container_scratch.secondary_extents.capacity(),
            workspace.container_scratch.column_counts.capacity(),
        )
    })
}
