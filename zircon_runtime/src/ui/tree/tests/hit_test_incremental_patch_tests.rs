use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    surface::UiArrangedNode,
    tree::{UiPointerEvents, UiVisibility},
};

#[test]
fn missing_ephemeral_lookup_requires_explicit_reindex() {
    let node_id = UiNodeId::new(1);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.lookup-reindex"),
        roots: vec![node_id].into(),
        nodes: vec![pointer_node(node_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0))].into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged(&arranged_tree);
    index.entry_cells.clear();
    index.entry_indices.clear();

    assert!(index.entry_by_node_id(node_id).is_none());
    index.ensure_entry_lookup();
    assert_eq!(
        index.entry_by_node_id(node_id).map(|entry| entry.node_id),
        Some(node_id)
    );
}

#[test]
fn bounded_cell_projection_is_lazy_and_row_major() {
    let cells = bounded_cells_for_frame(
        UiFrame::new(0.0, 0.0, 128.0, 128.0),
        2,
        2,
        64.0,
        UiFrame::new(0.0, 0.0, 128.0, 128.0),
    )
    .collect::<Vec<_>>();
    assert_eq!(cells, vec![0, 1, 2, 3]);
    assert!(bounded_cells_for_frame(
        UiFrame::new(0.0, 0.0, 128.0, 128.0),
        2,
        2,
        64.0,
        UiFrame::new(f32::NAN, 0.0, 1.0, 1.0),
    )
    .next()
    .is_none());
}

#[test]
fn moving_entry_across_cells_keeps_one_hit_index() {
    let moving_id = UiNodeId::new(1);
    let anchor_id = UiNodeId::new(2);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.incremental.cross-cell"),
        roots: vec![moving_id, anchor_id].into(),
        nodes: vec![
            pointer_node(moving_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0)),
            pointer_node(anchor_id, 1, UiFrame::new(100.0, 0.0, 20.0, 20.0)),
        ]
        .into(),
        draw_order: vec![moving_id, anchor_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(moving_id, 0), (anchor_id, 1)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged(&arranged_tree);

    arranged_tree.nodes[0].frame = UiFrame::new(70.0, 0.0, 20.0, 20.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
    assert_eq!(
        index.patch_arranged_geometry(&arranged_tree, &BTreeSet::from([moving_id]), &node_indices,),
        Ok(true)
    );

    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(5.0, 5.0))
            .top_hit,
        None
    );
    let moved_hit = index.hit_test_arranged(&arranged_tree, UiPoint::new(75.0, 5.0));
    assert_eq!(moved_hit.top_hit, Some(moving_id));
    assert_eq!(moved_hit.stacked, vec![moving_id]);
    let moving_entry_index = index.entry_indices[&moving_id];
    assert_eq!(
        index
            .grid
            .cells
            .iter()
            .flat_map(|cell| cell.entries.iter())
            .filter(|entry_index| **entry_index == moving_entry_index)
            .count(),
        1
    );
}

#[test]
fn geometry_patch_reuses_route_table() {
    let node_id = UiNodeId::new(5);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.geometry-route-reuse"),
        roots: vec![node_id].into(),
        nodes: vec![pointer_node(node_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0))].into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(node_id, 0)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged(&arranged_tree);
    let route_nodes = index.grid.route_nodes.clone();

    arranged_tree.nodes[0].frame = UiFrame::new(10.0, 0.0, 20.0, 20.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
    assert_eq!(
        index.patch_arranged_geometry(&arranged_tree, &BTreeSet::from([node_id]), &node_indices,),
        Ok(true)
    );
    assert!(std::sync::Arc::ptr_eq(
        &route_nodes,
        &index.grid.route_nodes
    ));
}

#[test]
fn malformed_parent_route_fails_closed() {
    let parent_id = UiNodeId::new(6);
    let child_id = UiNodeId::new(7);
    let frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
    let mut parent = pointer_node(parent_id, 0, frame);
    parent.children.push(child_id);
    let mut child = pointer_node(child_id, 1, frame);
    child.parent = Some(parent_id);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.malformed-parent-route"),
        roots: vec![parent_id].into(),
        nodes: vec![parent, child].into(),
        draw_order: vec![parent_id, child_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged(&arranged_tree);
    let parent_route_index = index.grid.entries[0].route_node_index as usize;
    std::sync::Arc::make_mut(&mut index.grid.route_nodes)[parent_route_index].route_valid = false;

    let hit = index.hit_test_arranged(&arranged_tree, UiPoint::new(5.0, 5.0));

    assert_eq!(hit.top_hit, None);
    assert_eq!(hit.top_entry_index, None);
    assert!(hit.stacked.is_empty());
    assert!(hit.path.has_consistent_route());
}

#[test]
fn self_none_excludes_the_node_but_keeps_pointer_children() {
    let parent_id = UiNodeId::new(10);
    let child_id = UiNodeId::new(11);
    let frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
    let mut parent = pointer_node(parent_id, 0, frame);
    parent.children.push(child_id);
    parent.pointer_events = UiPointerEvents::SelfNone;
    let mut child = pointer_node(child_id, 1, frame);
    child.parent = Some(parent_id);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.pointer-events.self-none"),
        roots: vec![parent_id].into(),
        nodes: vec![parent, child].into(),
        draw_order: vec![parent_id, child_id].into(),
        canvas_layers: Vec::new().into(),
    };

    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged(&arranged_tree);

    assert_eq!(
        index
            .grid
            .entries
            .iter()
            .map(|entry| entry.node_id)
            .collect::<Vec<_>>(),
        vec![child_id]
    );
}

#[test]
fn none_excludes_the_entire_pointer_subtree() {
    let parent_id = UiNodeId::new(20);
    let child_id = UiNodeId::new(21);
    let frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
    let mut parent = pointer_node(parent_id, 0, frame);
    parent.children.push(child_id);
    parent.pointer_events = UiPointerEvents::None;
    let mut child = pointer_node(child_id, 1, frame);
    child.parent = Some(parent_id);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.pointer-events.none"),
        roots: vec![parent_id].into(),
        nodes: vec![parent, child].into(),
        draw_order: vec![parent_id, child_id].into(),
        canvas_layers: Vec::new().into(),
    };

    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged(&arranged_tree);

    assert!(index.grid.entries.is_empty());
}

#[test]
fn hit_grid_bounds_geometry_and_cell_count_are_bounded() {
    let valid_id = UiNodeId::new(30);
    let invalid_id = UiNodeId::new(31);
    let huge_id = UiNodeId::new(32);
    let valid_frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
    let huge_frame = UiFrame::new(0.0, 0.0, 1_000_000.0, 1_000_000.0);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.bounded-grid"),
        roots: vec![valid_id, invalid_id, huge_id].into(),
        nodes: vec![
            pointer_node(valid_id, 0, valid_frame),
            pointer_node(invalid_id, 1, UiFrame::new(f32::NAN, 0.0, 20.0, 20.0)),
            pointer_node(huge_id, 2, huge_frame),
        ]
        .into(),
        draw_order: vec![valid_id, invalid_id, huge_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(valid_id, 0), (invalid_id, 1), (huge_id, 2)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

    assert!(index.grid.columns > 0);
    assert!(index.grid.rows > 0);
    assert!((index.grid.columns as usize) * (index.grid.rows as usize) <= HIT_GRID_MAX_CELL_COUNT);
    assert_eq!(index.grid.columns, 1);
    assert_eq!(index.grid.rows, 1);
    assert!(index
        .grid
        .entries
        .iter()
        .all(|entry| entry.node_id != invalid_id));
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
            .top_hit,
        Some(huge_id)
    );
}

#[test]
fn ordinary_bounds_keep_fine_grained_cell_partitioning() {
    let first_id = UiNodeId::new(40);
    let second_id = UiNodeId::new(41);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.fine-grid"),
        roots: vec![first_id, second_id].into(),
        nodes: vec![
            pointer_node(first_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0)),
            pointer_node(second_id, 1, UiFrame::new(128.0, 0.0, 20.0, 20.0)),
        ]
        .into(),
        draw_order: vec![first_id, second_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(first_id, 0), (second_id, 1)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

    assert!(index.grid.columns >= 2);
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
            .top_hit,
        Some(first_id)
    );
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(138.0, 10.0))
            .top_hit,
        Some(second_id)
    );
}

#[test]
fn capacity_envelope_absorbs_small_growth_and_regrids_only_at_geometric_boundaries() {
    let root_id = UiNodeId::new(50);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.capacity-envelope"),
        roots: vec![root_id].into(),
        nodes: vec![pointer_node(
            root_id,
            0,
            UiFrame::new(0.0, 0.0, 120.0, 60.0),
        )]
        .into(),
        draw_order: vec![root_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(root_id, 0)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

    assert_eq!(index.grid.bounds, UiFrame::new(0.0, 0.0, 128.0, 64.0));
    arranged_tree.nodes[0].frame = UiFrame::new(0.0, 0.0, 121.0, 60.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
    assert_eq!(
        index.patch_arranged_geometry(&arranged_tree, &BTreeSet::from([root_id]), &node_indices,),
        Ok(true)
    );
    assert_eq!(index.grid.bounds, UiFrame::new(0.0, 0.0, 128.0, 64.0));

    arranged_tree.nodes[0].frame = UiFrame::new(0.0, 0.0, 129.0, 60.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
    assert_eq!(
        index.patch_arranged_geometry(&arranged_tree, &BTreeSet::from([root_id]), &node_indices,),
        Err(())
    );
}

fn pointer_node(node_id: UiNodeId, paint_order: u64, frame: UiFrame) -> UiArrangedNode {
    UiArrangedNode {
        node_id,
        node_path: UiNodePath::new(format!("root/{}", node_id.0)),
        parent: None,
        children: Vec::new(),
        frame,
        clip_frame: frame,
        z_index: 0,
        paint_order,
        visibility: UiVisibility::Visible,
        input_policy: UiInputPolicy::Receive,
        pointer_events: Default::default(),
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: false,
        clip_to_bounds: false,
        control_id: None,
        slot: None,
    }
}
