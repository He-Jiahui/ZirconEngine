use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    tree::{UiPointerEvents, UiTreeNode},
};

#[test]
fn indexed_focus_path_matches_legacy_route_and_missing_node_fallback() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let leaf_id = UiNodeId::new(3);
    let missing_id = UiNodeId::new(4);
    let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
    let mut tree = UiTree::new(UiTreeId::new("ui.arranged.focus-path-index"));
    tree.insert_root(UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(frame));
    tree.insert_child(
        root_id,
        UiTreeNode::new(child_id, UiNodePath::new("root/child")).with_frame(frame),
    )
    .unwrap();
    tree.insert_child(
        child_id,
        UiTreeNode::new(leaf_id, UiNodePath::new("root/child/leaf")).with_frame(frame),
    )
    .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let node_indices = arranged_node_indices(&arranged_tree);
    assert_eq!(
        arranged_focus_path_indexed(&arranged_tree, &node_indices, Some(leaf_id)),
        arranged_focus_path(&arranged_tree, Some(leaf_id))
    );

    let missing = arranged_focus_path_indexed(&arranged_tree, &node_indices, Some(missing_id));
    assert_eq!(missing.focused, Some(missing_id));
    assert!(missing.bubble_route.is_empty());
}

#[test]
fn indexed_focus_path_validation_rejects_a_reparented_route() {
    let root_id = UiNodeId::new(1);
    let left_id = UiNodeId::new(2);
    let right_id = UiNodeId::new(3);
    let leaf_id = UiNodeId::new(4);
    let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
    let build_tree = |leaf_parent| {
        let mut tree = UiTree::new(UiTreeId::new("ui.arranged.focus-path-reparent"));
        tree.insert_root(UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(frame));
        tree.insert_child(
            root_id,
            UiTreeNode::new(left_id, UiNodePath::new("root/left")).with_frame(frame),
        )
        .unwrap();
        tree.insert_child(
            root_id,
            UiTreeNode::new(right_id, UiNodePath::new("root/right")).with_frame(frame),
        )
        .unwrap();
        tree.insert_child(
            leaf_parent,
            UiTreeNode::new(leaf_id, UiNodePath::new("root/leaf")).with_frame(frame),
        )
        .unwrap();
        tree
    };

    let before = build_tree(left_id);
    let before_arranged = build_arranged_tree(&before);
    let before_indices = arranged_node_indices(&before_arranged);
    let path = arranged_focus_path_indexed(&before_arranged, &before_indices, Some(leaf_id));
    assert!(arranged_focus_path_matches_indexed(
        &before_arranged,
        &before_indices,
        &path,
        Some(leaf_id),
    ));

    let after = build_tree(right_id);
    let after_arranged = build_arranged_tree(&after);
    let after_indices = arranged_node_indices(&after_arranged);
    assert!(!arranged_focus_path_matches_indexed(
        &after_arranged,
        &after_indices,
        &path,
        Some(leaf_id),
    ));
    assert_eq!(
        arranged_focus_path_indexed(&after_arranged, &after_indices, Some(leaf_id)).bubble_route,
        vec![leaf_id, right_id, root_id]
    );
}

#[test]
fn parent_input_patch_preserves_arranged_structure_allocations_for_descendants() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let leaf_id = UiNodeId::new(3);
    let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
    let mut tree = UiTree::new(UiTreeId::new("ui.arranged.input-patch"));
    let mut root = UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(frame);
    root.input_policy = UiInputPolicy::Receive;
    tree.insert_root(root);
    tree.insert_child(
        root_id,
        UiTreeNode::new(child_id, UiNodePath::new("root/child")).with_frame(frame),
    )
    .unwrap();
    tree.insert_child(
        child_id,
        UiTreeNode::new(leaf_id, UiNodePath::new("root/child/leaf")).with_frame(frame),
    )
    .unwrap();

    let mut arranged_tree = build_arranged_tree(&tree);
    let node_indices = arranged_node_indices(&arranged_tree);
    let slot_indices = arranged_slot_indices(&tree);
    let root_index = node_indices[&root_id];
    let child_index = node_indices[&child_id];
    let root_path_ptr = arranged_tree.nodes[root_index].node_path.0.as_ptr();
    let root_children_ptr = arranged_tree.nodes[root_index].children.as_ptr();
    let child_path_ptr = arranged_tree.nodes[child_index].node_path.0.as_ptr();
    let child_children_ptr = arranged_tree.nodes[child_index].children.as_ptr();

    tree.node_mut(root_id).unwrap().input_policy = UiInputPolicy::Ignore;
    tree.node_mut(root_id).unwrap().pointer_events = UiPointerEvents::None;
    tree.node_mut(child_id).unwrap().state_flags.clickable = true;
    let affected = patch_arranged_tree_input(
        &tree,
        &mut arranged_tree,
        &BTreeSet::from([root_id]),
        &BTreeSet::new(),
        &node_indices,
        &slot_indices,
    )
    .expect("input-only mutation should stay on the incremental path");

    assert_eq!(affected, BTreeSet::from([root_id, child_id, leaf_id]));
    assert_eq!(
        arranged_tree.nodes[root_index].node_path.0.as_ptr(),
        root_path_ptr
    );
    assert_eq!(
        arranged_tree.nodes[root_index].children.as_ptr(),
        root_children_ptr
    );
    assert_eq!(
        arranged_tree.nodes[child_index].node_path.0.as_ptr(),
        child_path_ptr
    );
    assert_eq!(
        arranged_tree.nodes[child_index].children.as_ptr(),
        child_children_ptr
    );
    assert_eq!(
        arranged_tree.nodes[root_index].input_policy,
        UiInputPolicy::Ignore
    );
    assert!(arranged_tree.nodes[child_index].clickable);
    assert_eq!(
        arranged_effective_input_policy_indexed(&arranged_tree, &node_indices, leaf_id,).unwrap(),
        UiInputPolicy::Ignore
    );
    assert!(
        !is_arranged_child_hit_path_visible_indexed(&arranged_tree, &node_indices, leaf_id,)
            .unwrap()
    );
}

#[test]
fn clipping_geometry_patch_returns_the_exact_affected_subtree() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let sibling_id = UiNodeId::new(3);
    let mut tree = UiTree::new(UiTreeId::new("ui.arranged.clipping-geometry-patch"));
    let mut root = UiTreeNode::new(root_id, UiNodePath::new("root"))
        .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0));
    root.clip_to_bounds = true;
    tree.insert_root(root);
    tree.insert_child(
        root_id,
        UiTreeNode::new(child_id, UiNodePath::new("root/child"))
            .with_frame(UiFrame::new(80.0, 0.0, 40.0, 20.0)),
    )
    .unwrap();
    tree.insert_root(
        UiTreeNode::new(sibling_id, UiNodePath::new("sibling"))
            .with_frame(UiFrame::new(200.0, 0.0, 20.0, 20.0)),
    );

    let mut arranged_tree = build_arranged_tree(&tree);
    let node_indices = arranged_node_indices(&arranged_tree);
    let slot_indices = arranged_slot_indices(&tree);
    tree.node_mut(root_id).unwrap().layout_cache.frame = UiFrame::new(0.0, 0.0, 90.0, 100.0);

    let affected = patch_arranged_tree_geometry(
        &tree,
        &mut arranged_tree,
        &BTreeSet::from([root_id]),
        &BTreeSet::from([root_id, child_id]),
        &node_indices,
        &slot_indices,
    )
    .expect("clipping geometry should patch its retained subtree");

    assert_eq!(affected, BTreeSet::from([root_id, child_id]));
    let child = arranged_node_indexed(&arranged_tree, &node_indices, child_id).unwrap();
    assert_eq!(child.clip_frame, UiFrame::new(0.0, 0.0, 90.0, 100.0));
    assert_eq!(
        child.frame.intersection(child.clip_frame),
        Some(UiFrame::new(80.0, 0.0, 10.0, 20.0))
    );
    assert_eq!(
        arranged_node_indexed(&arranged_tree, &node_indices, sibling_id)
            .unwrap()
            .clip_frame,
        UiFrame::new(200.0, 0.0, 20.0, 20.0)
    );
}

#[test]
fn input_patch_allows_geometry_committed_by_the_following_patch() {
    let root_id = UiNodeId::new(1);
    let original_frame = UiFrame::new(0.0, 0.0, 100.0, 60.0);
    let resized_frame = UiFrame::new(0.0, 0.0, 140.0, 80.0);
    let mut tree = UiTree::new(UiTreeId::new("ui.arranged.input-geometry-patch"));
    tree.insert_root(UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(original_frame));
    let mut arranged_tree = build_arranged_tree(&tree);
    let node_indices = arranged_node_indices(&arranged_tree);
    let slot_indices = arranged_slot_indices(&tree);
    let root = tree.node_mut(root_id).expect("root should exist");
    root.layout_cache.frame = resized_frame;
    root.state_flags.enabled = false;

    let input_affected = patch_arranged_tree_input(
        &tree,
        &mut arranged_tree,
        &BTreeSet::from([root_id]),
        &BTreeSet::from([root_id]),
        &node_indices,
        &slot_indices,
    )
    .expect("input patch should admit geometry owned by the following patch");
    let input_patched = arranged_node_indexed(&arranged_tree, &node_indices, root_id).unwrap();
    assert!(!input_patched.enabled);
    assert_eq!(input_patched.frame, original_frame);

    let geometry_affected = patch_arranged_tree_geometry(
        &tree,
        &mut arranged_tree,
        &BTreeSet::from([root_id]),
        &BTreeSet::from([root_id]),
        &node_indices,
        &slot_indices,
    )
    .expect("geometry patch should validate the combined transaction");

    assert_eq!(input_affected, BTreeSet::from([root_id]));
    assert_eq!(geometry_affected, BTreeSet::from([root_id]));
    let committed = arranged_node_indexed(&arranged_tree, &node_indices, root_id).unwrap();
    assert!(!committed.enabled);
    assert_eq!(committed.frame, resized_frame);
}
