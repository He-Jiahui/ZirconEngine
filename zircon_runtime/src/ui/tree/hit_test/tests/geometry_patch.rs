use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::{UiFrame, UiPoint},
    tree::{UiInputPolicy, UiVisibility},
};

#[test]
fn reverse_cell_index_reuses_capacity_for_stable_entry_ids() {
    let node_id = UiNodeId::new(1);
    let wide = UiFrame::new(0.0, 0.0, 16_384.0, 64.0);
    let small = UiFrame::new(0.0, 0.0, 8.0, 8.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.reverse-map-capacity-reuse"),
        roots: vec![node_id].into(),
        nodes: vec![pointer_node(node_id, 0, wide, wide)].into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(node_id, 0)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);
    let retained_capacity = index.entry_cells[&node_id].capacity();
    assert!(retained_capacity >= 64);

    arranged_tree.nodes[0].frame = small;
    arranged_tree.nodes[0].clip_frame = small;
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

    let cells = &index.entry_cells[&node_id];
    assert_eq!(cells.len(), 1);
    assert_eq!(cells.capacity(), retained_capacity);
}

#[test]
fn geometry_patch_activates_and_deactivates_stable_entry_cells() {
    let anchor_id = UiNodeId::new(1);
    let moving_id = UiNodeId::new(2);
    let anchor_frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.stable-clipped-entry"),
        roots: vec![anchor_id, moving_id].into(),
        nodes: vec![
            pointer_node(anchor_id, 0, anchor_frame, anchor_frame),
            pointer_node(
                moving_id,
                1,
                UiFrame::new(200.0, 0.0, 20.0, 20.0),
                anchor_frame,
            ),
        ]
        .into(),
        draw_order: vec![anchor_id, moving_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(anchor_id, 0), (moving_id, 1)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

    assert_eq!(index.grid.entries.len(), 2);
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
            .top_hit,
        Some(anchor_id)
    );

    arranged_tree.nodes[1].frame = UiFrame::new(5.0, 5.0, 20.0, 20.0);
    #[cfg(feature = "profiling")]
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    #[cfg(feature = "profiling")]
    {
        let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
        config.session_id = "ui04-base-cell-membership-patch".to_owned();
        config.max_counters = 64;
        assert!(crate::core::diagnostics::profiling::start_capture(config).active);
    }
    assert!(index
        .patch_arranged_geometry(&arranged_tree, &BTreeSet::from([moving_id]), &node_indices,)
        .unwrap());
    #[cfg(feature = "profiling")]
    {
        let profile = crate::core::diagnostics::profiling::snapshot();
        assert!(!crate::core::diagnostics::profiling::reset_capture().active);
        for suffix in [
            "cell_patch_removal_count",
            "cell_patch_addition_count",
            "cell_patch_materialized_membership_count",
            "cell_patch_replacement_buffer_count",
        ] {
            let base = format!("ui.hit_grid.{suffix}");
            let projected = format!("ui.surface_projected_hit.{suffix}");
            let base_counter = profile
                .counters
                .iter()
                .find(|counter| counter.stream == "runtime" && counter.name == base)
                .unwrap_or_else(|| panic!("base patch should record {base}"));
            if suffix == "cell_patch_removal_count" {
                assert_eq!(base_counter.value, 0.0);
            } else {
                assert!(
                    base_counter.value > 0.0,
                    "base patch should record positive {base}"
                );
            }
            assert!(
                !profile
                    .counters
                    .iter()
                    .any(|counter| { counter.stream == "runtime" && counter.name == projected }),
                "base patch should not record projected {projected}"
            );
        }
    }
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
            .top_hit,
        Some(moving_id)
    );

    arranged_tree.nodes[1].frame = UiFrame::new(200.0, 0.0, 20.0, 20.0);
    assert!(index
        .patch_arranged_geometry(&arranged_tree, &BTreeSet::from([moving_id]), &node_indices,)
        .unwrap());
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
            .top_hit,
        Some(anchor_id)
    );
}

#[test]
fn dense_geometry_batches_match_full_rebuild_and_visit_each_cell_once() {
    for entry_count in [1_000, 10_000] {
        for changed_count in [0, 1, 64] {
            let source_frame = UiFrame::new(1.0, 1.0, 8.0, 8.0);
            let target_frame = UiFrame::new(80.0, 1.0, 8.0, 8.0);
            let anchor_id = UiNodeId::new(entry_count as u64 + 1);
            let mut nodes = (0..entry_count)
                .map(|entry_index| {
                    pointer_node(
                        UiNodeId::new(entry_index as u64 + 1),
                        entry_index as u64,
                        source_frame,
                        source_frame,
                    )
                })
                .collect::<Vec<_>>();
            nodes.push(pointer_node(
                anchor_id,
                entry_count as u64,
                target_frame,
                target_frame,
            ));
            let draw_order = nodes.iter().map(|node| node.node_id).collect::<Vec<_>>();
            let node_indices = draw_order
                .iter()
                .enumerate()
                .map(|(index, node_id)| (*node_id, index))
                .collect::<BTreeMap<_, _>>();
            let mut arranged_tree = UiArrangedTree {
                tree_id: UiTreeId::new("ui.hit.dense-geometry-batch"),
                roots: draw_order.clone().into(),
                nodes: nodes.into(),
                draw_order: draw_order.into(),
                canvas_layers: Vec::new().into(),
            };
            let mut index = UiHitTestIndex::default();
            index.rebuild_arranged_indexed(&arranged_tree, &node_indices);
            let before = index.clone();
            let changed = (0..changed_count)
                .map(|entry_index| UiNodeId::new(entry_index as u64 + 1))
                .collect::<BTreeSet<_>>();
            for entry_index in 0..changed_count {
                arranged_tree.nodes[entry_index].frame = target_frame;
                arranged_tree.nodes[entry_index].clip_frame = target_frame;
            }

            let (did_change, stats) = index
                .patch_arranged_geometry_with_stats(&arranged_tree, &changed, &node_indices)
                .unwrap();

            if changed_count == 0 {
                assert!(!did_change);
                assert_eq!(stats, UiCellMembershipPatchStats::default());
                assert_eq!(index, before);
                continue;
            }
            assert!(did_change);
            assert_eq!(stats.staged_cell_count, 2);
            assert_eq!(stats.published_cell_count, 2);
            assert_eq!(stats.source_membership_count, entry_count + 1);
            assert_eq!(stats.staged_removal_count, changed_count);
            assert_eq!(stats.staged_addition_count, changed_count);

            let mut rebuilt = UiHitTestIndex::default();
            rebuilt.rebuild_arranged_indexed(&arranged_tree, &node_indices);
            assert_eq!(index.grid, rebuilt.grid);
            for point in [UiPoint::new(4.0, 4.0), UiPoint::new(84.0, 4.0)] {
                assert_eq!(
                    index.hit_test_arranged(&arranged_tree, point),
                    rebuilt.hit_test_arranged(&arranged_tree, point)
                );
            }
        }
    }
}

#[test]
fn geometry_batch_preflight_failure_preserves_index() {
    let node_id = UiNodeId::new(1);
    let frame = UiFrame::new(0.0, 0.0, 8.0, 8.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.geometry-preflight"),
        roots: vec![node_id].into(),
        nodes: vec![pointer_node(node_id, 0, frame, frame)].into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(node_id, 0)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);
    let before = index.clone();
    arranged_tree.nodes[0].frame = UiFrame::new(1_000.0, 0.0, 8.0, 8.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;

    assert_eq!(
        index.patch_arranged_geometry(&arranged_tree, &BTreeSet::from([node_id]), &node_indices),
        Err(())
    );
    assert_eq!(index, before);
}

#[test]
fn cold_lookup_preflight_failure_preserves_reverse_maps_and_grid() {
    let node_id = UiNodeId::new(1);
    let frame = UiFrame::new(0.0, 0.0, 8.0, 8.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.cold-lookup-preflight"),
        roots: vec![node_id].into(),
        nodes: vec![pointer_node(node_id, 0, frame, frame)].into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(node_id, 0)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);
    index.entry_cells.clear();
    index.entry_indices.clear();
    let before_grid = index.grid.clone();
    let before_entry_cells = index.entry_cells.clone();
    let before_entry_indices = index.entry_indices.clone();

    arranged_tree.nodes[0].frame = UiFrame::new(1_000.0, 0.0, 8.0, 8.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;

    assert_eq!(
        index.patch_arranged_geometry(&arranged_tree, &BTreeSet::from([node_id]), &node_indices,),
        Err(())
    );
    assert_eq!(index.grid, before_grid);
    assert_eq!(index.entry_cells, before_entry_cells);
    assert_eq!(index.entry_indices, before_entry_indices);
}

#[test]
fn painter_reorder_then_cell_move_removes_nonmonotonic_entry_index() {
    let first_id = UiNodeId::new(1);
    let second_id = UiNodeId::new(2);
    let anchor_id = UiNodeId::new(3);
    let source_frame = UiFrame::new(1.0, 1.0, 8.0, 8.0);
    let target_frame = UiFrame::new(80.0, 1.0, 8.0, 8.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.reorder-then-move"),
        roots: vec![first_id, second_id, anchor_id].into(),
        nodes: vec![
            pointer_node(first_id, 0, source_frame, source_frame),
            pointer_node(second_id, 1, source_frame, source_frame),
            pointer_node(anchor_id, 2, target_frame, target_frame),
        ]
        .into(),
        draw_order: vec![first_id, second_id, anchor_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(first_id, 0), (second_id, 1), (anchor_id, 2)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

    arranged_tree.nodes[0].paint_order = 3;
    assert!(index
        .patch_arranged_geometry(&arranged_tree, &BTreeSet::from([first_id]), &node_indices,)
        .unwrap());
    assert_eq!(
        index
            .hit_test_arranged(&arranged_tree, UiPoint::new(4.0, 4.0))
            .stacked,
        vec![first_id, second_id]
    );
    let source_cell = index.entry_cells[&first_id][0];
    assert_eq!(index.grid.cells[source_cell].entries.as_slice(), &[1, 0]);

    arranged_tree.nodes[0].frame = target_frame;
    arranged_tree.nodes[0].clip_frame = target_frame;
    assert!(index
        .patch_arranged_geometry(&arranged_tree, &BTreeSet::from([first_id]), &node_indices,)
        .unwrap());
    assert_eq!(index.grid.cells[source_cell].entries.as_slice(), &[1]);
    let mut rebuilt = UiHitTestIndex::default();
    rebuilt.rebuild_arranged_indexed(&arranged_tree, &node_indices);
    for point in [UiPoint::new(4.0, 4.0), UiPoint::new(84.0, 4.0)] {
        let patched = index.hit_test_arranged(&arranged_tree, point);
        let expected = rebuilt.hit_test_arranged(&arranged_tree, point);
        assert_eq!(patched.top_hit, expected.top_hit);
        assert_eq!(patched.stacked, expected.stacked);
        assert_eq!(patched.path, expected.path);
    }
}

#[test]
fn geometry_scale_across_multiple_cells_matches_rebuild() {
    let scaled_id = UiNodeId::new(1);
    let anchor_id = UiNodeId::new(2);
    let source = UiFrame::new(1.0, 1.0, 8.0, 8.0);
    let scaled = UiFrame::new(64.0, 1.0, 120.0, 8.0);
    let anchor = UiFrame::new(192.0, 1.0, 8.0, 8.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.geometry-scale-cells"),
        roots: vec![scaled_id, anchor_id].into(),
        nodes: vec![
            pointer_node(scaled_id, 0, source, source),
            pointer_node(anchor_id, 1, anchor, anchor),
        ]
        .into(),
        draw_order: vec![scaled_id, anchor_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(scaled_id, 0), (anchor_id, 1)]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &node_indices);
    arranged_tree.nodes[0].frame = scaled;
    arranged_tree.nodes[0].clip_frame = scaled;

    let (changed, stats) = index
        .patch_arranged_geometry_with_stats(
            &arranged_tree,
            &BTreeSet::from([scaled_id]),
            &node_indices,
        )
        .unwrap();
    assert!(changed);
    assert!(stats.staged_cell_count >= 2);

    let mut rebuilt = UiHitTestIndex::default();
    rebuilt.rebuild_arranged_indexed(&arranged_tree, &node_indices);
    assert_eq!(index.grid, rebuilt.grid);
}

#[test]
fn input_patch_geometry_failure_preserves_route_and_complete_index() {
    let node_id = UiNodeId::new(1);
    let parent_id = UiNodeId::new(2);
    let frame = UiFrame::new(1.0, 1.0, 8.0, 8.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.input-atomicity"),
        roots: vec![node_id, parent_id].into(),
        nodes: vec![
            pointer_node(node_id, 0, frame, frame),
            pointer_node(parent_id, 1, frame, frame),
        ]
        .into(),
        draw_order: vec![node_id, parent_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let indices = BTreeMap::from([(node_id, 0), (parent_id, 1)]);
    let changed = BTreeSet::from([node_id]);
    let mut index = UiHitTestIndex::default();
    index.rebuild_arranged_indexed(&arranged_tree, &indices);
    let before = index.clone();
    arranged_tree.nodes[0].parent = Some(parent_id);
    arranged_tree.nodes[1].children.push(node_id);
    arranged_tree.roots = vec![parent_id].into();
    arranged_tree.nodes[0].frame = UiFrame::new(1_000.0, 1.0, 8.0, 8.0);
    arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
    let mut staged_routes = before.grid.route_nodes.clone();
    assert!(super::super::patch_route_nodes(
        &mut staged_routes,
        &arranged_tree,
        &changed,
        &indices
    )
    .unwrap());
    assert!(staged_routes[0].route_valid);

    assert_eq!(
        index.patch_arranged_input(&arranged_tree, &changed, &indices),
        Err(())
    );
    assert_eq!(index.grid, before.grid);
    assert_eq!(index.entry_cells, before.entry_cells);
    assert_eq!(index.entry_indices, before.entry_indices);
    assert!(std::sync::Arc::ptr_eq(
        &index.grid.route_nodes,
        &before.grid.route_nodes
    ));

    arranged_tree.nodes[0].frame = frame;
    arranged_tree.nodes[0].clip_frame = frame;
    assert_eq!(
        index.patch_arranged_input(&arranged_tree, &changed, &indices),
        Ok(true)
    );
    assert_eq!(index.grid.route_nodes, staged_routes);
}

fn pointer_node(
    node_id: UiNodeId,
    paint_order: u64,
    frame: UiFrame,
    clip_frame: UiFrame,
) -> UiArrangedNode {
    UiArrangedNode {
        node_id,
        node_path: UiNodePath::new(format!("root/{}", node_id.0)),
        parent: None,
        children: Vec::new(),
        frame,
        clip_frame,
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
