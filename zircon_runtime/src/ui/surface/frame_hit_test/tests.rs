use super::*;

#[test]
fn incremental_patch_source_does_not_scan_all_base_entries() {
    let source = include_str!("../frame_hit_test.rs");
    let patch_body = source
        .split_once("    fn patch(")
        .and_then(|(_, remainder)| remainder.split_once("\n    fn synchronize("))
        .map(|(body, _)| body)
        .expect("projected hit-test patch body should remain source-guardable");
    let compact_patch = patch_body
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let forbidden_global_scan = ["base_index", ".grid", ".entries", ".iter()"].concat();

    assert!(!compact_patch.contains(&forbidden_global_scan));
}

#[test]
fn affine_projection_maps_frame_and_clip_with_non_uniform_scale() {
    let source = UiFrame::new(10.0, 20.0, 40.0, 20.0);
    let target = UiFrame::new(100.0, 200.0, 80.0, 60.0);

    assert_eq!(
        project_frame(UiFrame::new(20.0, 25.0, 10.0, 5.0), source, target),
        UiFrame::new(120.0, 215.0, 20.0, 15.0)
    );
    assert_eq!(
        project_frame(UiFrame::new(15.0, 22.0, 20.0, 10.0), source, target),
        UiFrame::new(110.0, 206.0, 40.0, 30.0)
    );
}

#[test]
fn incremental_z_crossing_overlay_base_falls_back_to_projected_rebuild() {
    let popup_root = UiNodeId::new(10);
    let popup_entry = hit_entry(UiNodeId::new(11), popup_root, 1, 0);
    let mut ordinary_entry = hit_entry(UiNodeId::new(30), UiNodeId::new(30), 0, 0);
    let base_grid = test_hit_grid(vec![
        (ordinary_entry.clone(), ordinary_entry.node_id),
        (popup_entry.clone(), popup_root),
    ]);
    let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let projections = [UiHitTestProjection {
        popup_root,
        source_frame: frame,
        target_frame: Some(frame),
        target_clip: Some(frame),
        stack_order: 0,
    }];
    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(&base_grid, &projections);
    assert_eq!(projected.overlay_z_base, 2);

    ordinary_entry.z_index = 100;
    let changed_node_ids = BTreeSet::from([ordinary_entry.node_id]);
    let updated_base = UiHitTestIndex::from_grid(build_projected_grid(
        &base_grid,
        vec![ordinary_entry, popup_entry.clone()],
    ));
    projected.synchronize(&updated_base, &projections, &changed_node_ids, false);

    assert_eq!(projected.overlay_z_base, 101);
    let hit = hit_test_projected_grid_with_query(
        &projected.grid,
        &UiArrangedTree::default(),
        UiHitTestQuery::new(UiPoint::new(5.0, 5.0)),
    );
    assert_eq!(hit.top_hit, Some(popup_entry.node_id));
}

#[test]
fn base_full_rebuild_refreshes_same_count_non_projected_entries() {
    let popup_root = UiNodeId::new(10);
    let popup_entry = hit_entry(UiNodeId::new(11), popup_root, 5, 0);
    let mut ordinary_entry = hit_entry(UiNodeId::new(30), UiNodeId::new(30), 0, 0);
    ordinary_entry.frame = UiFrame::new(20.0, 0.0, 10.0, 10.0);
    ordinary_entry.clip_frame = ordinary_entry.frame;
    let base_grid = test_hit_grid(vec![
        (popup_entry.clone(), popup_root),
        (ordinary_entry.clone(), ordinary_entry.node_id),
    ]);
    let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let projections = [UiHitTestProjection {
        popup_root,
        source_frame: frame,
        target_frame: Some(UiFrame::new(100.0, 0.0, 10.0, 10.0)),
        target_clip: Some(UiFrame::new(100.0, 0.0, 10.0, 10.0)),
        stack_order: 0,
    }];
    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(&base_grid, &projections);

    ordinary_entry.frame = UiFrame::new(40.0, 0.0, 10.0, 10.0);
    ordinary_entry.clip_frame = ordinary_entry.frame;
    let rebuilt_base = UiHitTestIndex::from_grid(test_hit_grid(vec![
        (popup_entry, popup_root),
        (ordinary_entry.clone(), UiNodeId::new(31)),
    ]));

    projected.synchronize(&rebuilt_base, &projections, &BTreeSet::new(), true);

    let refreshed = projected
        .grid
        .entries
        .iter()
        .find(|entry| entry.node_id == ordinary_entry.node_id)
        .expect("same-count base rebuild must keep the ordinary entry");
    assert_eq!(refreshed.frame, ordinary_entry.frame);
    assert_eq!(refreshed.clip_frame, ordinary_entry.clip_frame);
    let route = rebuilt_base.grid.route_nodes[refreshed.route_node_index as usize];
    assert_eq!(
        route
            .parent_index()
            .and_then(|index| rebuilt_base.grid.route_nodes.get(index))
            .map(|parent| parent.node_id),
        Some(UiNodeId::new(31))
    );
}

#[test]
fn incremental_projection_refreshes_rendered_target_clip() {
    let popup_root = UiNodeId::new(10);
    let popup_entry = hit_entry(UiNodeId::new(11), popup_root, 5, 0);
    let base_grid = test_hit_grid(vec![(popup_entry.clone(), popup_root)]);
    let base_index = UiHitTestIndex::from_grid(base_grid.clone());
    let source_frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(
        &base_grid,
        &[UiHitTestProjection {
            popup_root,
            source_frame,
            target_frame: Some(source_frame),
            target_clip: Some(source_frame),
            stack_order: 0,
        }],
    );

    let clipped_frame = UiFrame::new(2.0, 2.0, 4.0, 4.0);
    projected.synchronize(
        &base_index,
        &[UiHitTestProjection {
            popup_root,
            source_frame,
            target_frame: Some(source_frame),
            target_clip: Some(clipped_frame),
            stack_order: 0,
        }],
        &BTreeSet::new(),
        false,
    );

    let refreshed = projected
        .grid
        .entries
        .iter()
        .find(|entry| entry.node_id == popup_entry.node_id)
        .expect("projected popup entry should remain indexed");
    assert_eq!(refreshed.frame, source_frame);
    assert_eq!(refreshed.clip_frame, clipped_frame);
    assert_eq!(
        hit_test_projected_grid_with_query(
            &projected.grid,
            &UiArrangedTree::default(),
            UiHitTestQuery::new(UiPoint::new(1.0, 1.0)),
        )
        .top_hit,
        None
    );
    assert_eq!(
        hit_test_projected_grid_with_query(
            &projected.grid,
            &UiArrangedTree::default(),
            UiHitTestQuery::new(UiPoint::new(3.0, 3.0)),
        )
        .top_hit,
        Some(popup_entry.node_id)
    );
}

#[test]
fn projected_order_preserves_inner_z_and_places_next_popup_above_entire_subtree() {
    let first_popup = UiNodeId::new(10);
    let second_popup = UiNodeId::new(20);
    let low_z_high_paint = hit_entry(UiNodeId::new(11), first_popup, 5, 100);
    let high_z_low_paint = hit_entry(UiNodeId::new(12), first_popup, 6, 0);
    let next_popup_low_z = hit_entry(UiNodeId::new(21), second_popup, -100, 0);
    let base_grid = test_hit_grid(vec![
        (low_z_high_paint.clone(), first_popup),
        (high_z_low_paint.clone(), first_popup),
        (next_popup_low_z.clone(), second_popup),
    ]);
    let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let projections = [
        UiHitTestProjection {
            popup_root: first_popup,
            source_frame: frame,
            target_frame: Some(frame),
            target_clip: Some(frame),
            stack_order: 0,
        },
        UiHitTestProjection {
            popup_root: second_popup,
            source_frame: frame,
            target_frame: Some(frame),
            target_clip: Some(frame),
            stack_order: 1,
        },
    ];
    let projection_by_root = projection_by_root(&projections);
    let plan = projection_order_plan(&base_grid, &projection_by_root, 7);

    assert!(
        plan.order_keys[&low_z_high_paint.node_id] < plan.order_keys[&high_z_low_paint.node_id]
    );
    assert!(
        plan.order_keys[&high_z_low_paint.node_id] < plan.order_keys[&next_popup_low_z.node_id]
    );

    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(&base_grid, &projections);
    let hit = hit_test_projected_grid_with_query(
        &projected.grid,
        &UiArrangedTree::default(),
        UiHitTestQuery::new(UiPoint::new(5.0, 5.0)),
    );
    assert_eq!(hit.top_hit, Some(next_popup_low_z.node_id));
    let projected_z = |node_id| {
        projected
            .grid
            .entries
            .iter()
            .find(|entry| entry.node_id == node_id)
            .map(|entry| entry.z_index)
            .expect("projected entry should retain an explicit z layer")
    };
    assert!(projected_z(low_z_high_paint.node_id) < projected_z(high_z_low_paint.node_id));
    assert!(projected_z(high_z_low_paint.node_id) < projected_z(next_popup_low_z.node_id));
    assert_eq!(
        hit.stacked,
        vec![
            next_popup_low_z.node_id,
            high_z_low_paint.node_id,
            low_z_high_paint.node_id,
        ]
    );
}

#[test]
fn projected_grid_bounds_cells_and_rejects_non_finite_membership() {
    let popup_root = UiNodeId::new(10);
    let mut huge = hit_entry(UiNodeId::new(11), popup_root, 1, 0);
    huge.frame = UiFrame::new(0.0, 0.0, 1_000_000.0, 1_000_000.0);
    huge.clip_frame = huge.frame;
    let mut invalid = hit_entry(UiNodeId::new(12), popup_root, 2, 0);
    invalid.frame = UiFrame::new(f32::NAN, 0.0, 10.0, 10.0);
    invalid.clip_frame = invalid.frame;

    let grid = build_projected_grid(&UiHitTestGrid::default(), vec![huge, invalid.clone()]);

    assert_eq!((grid.columns, grid.rows), (64, 64));
    assert_eq!(grid.cells.len(), 4_096);
    assert!(grid.cells.iter().all(|cell| {
        cell.entries
            .iter()
            .all(|entry_index| grid.entries[*entry_index].node_id != invalid.node_id)
    }));
}

fn hit_entry(
    node_id: UiNodeId,
    _popup_root: UiNodeId,
    z_index: i32,
    paint_order: u64,
) -> UiHitTestEntry {
    UiHitTestEntry {
        node_id,
        frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
        clip_frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
        z_index,
        paint_order,
        control_id: None,
        route_node_index: u32::try_from(node_id.0).expect("test node id must fit route index"),
    }
}

fn test_hit_grid(entries: Vec<(UiHitTestEntry, UiNodeId)>) -> UiHitTestGrid {
    let max_node_id = entries
        .iter()
        .flat_map(|(entry, parent)| [entry.node_id.0, parent.0])
        .max()
        .unwrap_or_default();
    let mut route_nodes = (0..=max_node_id)
        .map(|node_id| UiHitRouteNode::invalid(UiNodeId::new(node_id)))
        .collect::<Vec<_>>();
    for (entry, parent_id) in &entries {
        let parent_index = u32::try_from(parent_id.0).expect("test parent id must fit route index");
        route_nodes[parent_index as usize] = UiHitRouteNode {
            node_id: *parent_id,
            parent_index: UiHitRouteNode::NO_PARENT_INDEX,
            effective_input_policy: UiInputPolicy::Receive,
            pointer_path_visible: true,
            descendant_pointer_path_visible: true,
            route_valid: true,
        };
        route_nodes[entry.route_node_index as usize] = UiHitRouteNode {
            node_id: entry.node_id,
            parent_index: if entry.node_id == *parent_id {
                UiHitRouteNode::NO_PARENT_INDEX
            } else {
                parent_index
            },
            effective_input_policy: UiInputPolicy::Receive,
            pointer_path_visible: true,
            descendant_pointer_path_visible: true,
            route_valid: true,
        };
    }
    let base_grid = UiHitTestGrid {
        route_nodes: std::sync::Arc::new(route_nodes),
        ..UiHitTestGrid::default()
    };
    build_projected_grid(
        &base_grid,
        entries.into_iter().map(|(entry, _)| entry).collect(),
    )
}
