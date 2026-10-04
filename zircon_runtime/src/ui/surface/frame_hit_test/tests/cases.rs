use super::*;

fn hit_test_projected_grid_with_query(
    grid: &UiHitTestGrid,
    arranged_tree: &UiArrangedTree,
    query: UiHitTestQuery,
) -> UiHitTestResult {
    UiHitTestIndex::hit_test_grid_arranged_with_query(grid, arranged_tree, query)
}

#[test]
fn incremental_patch_source_does_not_scan_all_base_entries() {
    let source = include_str!("../../frame_hit_test.rs");
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
fn popup_projection_reserves_popup_stack_capacity() {
    let source = include_str!("../../frame_hit_test.rs");
    let projection_body = source
        .split_once("    fn popup_hit_test_projections(&self) -> Vec<UiHitTestProjection> {")
        .and_then(|(_, remainder)| {
            remainder.split_once("\n    }\n}\n\npub(super) fn hit_test_surface_frame")
        })
        .map(|(body, _)| body)
        .expect("surface must keep a dedicated popup projection helper");

    assert!(projection_body.contains("let popup_stack = &self.input.popup_stack;"));
    assert!(projection_body.contains("Vec::with_capacity(popup_stack.len())"));
    assert!(projection_body.contains("for (stack_order, popup) in popup_stack.iter().enumerate()"));
    assert!(projection_body.contains("let Some(popup_root) = popup.popup_node else"));
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

#[test]
fn projected_dense_geometry_batches_match_full_rebuild_at_scale() {
    for entry_count in [1_000, 10_000] {
        for changed_count in [0, 1, 64] {
            let source_frame = UiFrame::new(1.0, 1.0, 8.0, 8.0);
            let target_frame = UiFrame::new(80.0, 1.0, 8.0, 8.0);
            let anchor_id = UiNodeId::new(entry_count as u64 + 1);
            let mut entries = (0..entry_count)
                .map(|entry_index| {
                    let node_id = UiNodeId::new(entry_index as u64 + 1);
                    let mut entry = hit_entry(node_id, node_id, 0, entry_index as u64);
                    entry.frame = source_frame;
                    entry.clip_frame = source_frame;
                    (entry, node_id)
                })
                .collect::<Vec<_>>();
            let mut anchor = hit_entry(anchor_id, anchor_id, 0, entry_count as u64);
            anchor.frame = target_frame;
            anchor.clip_frame = target_frame;
            entries.push((anchor, anchor_id));
            let base_grid = test_hit_grid(entries.clone());
            let mut projected = UiProjectedHitTestIndex::default();
            projected.rebuild(&base_grid, &[]);
            let before = projected.grid.clone();
            let changed_node_ids = (0..changed_count)
                .map(|entry_index| UiNodeId::new(entry_index as u64 + 1))
                .collect::<BTreeSet<_>>();
            for (entry, _) in entries.iter_mut().take(changed_count) {
                entry.frame = target_frame;
                entry.clip_frame = target_frame;
            }
            let updated_grid = test_hit_grid(entries);
            let updated_base = UiHitTestIndex::from_grid(updated_grid.clone());

            let changed = projected
                .patch(&updated_base, &[], &changed_node_ids)
                .unwrap();

            if changed_count == 0 {
                assert!(!changed);
                assert_eq!(projected.grid, before);
            } else {
                assert!(changed);
                let mut rebuilt = UiProjectedHitTestIndex::default();
                rebuilt.rebuild(&updated_grid, &[]);
                assert_eq!(projected.grid, rebuilt.grid);
                for point in [UiPoint::new(4.0, 4.0), UiPoint::new(84.0, 4.0)] {
                    assert_eq!(
                        hit_test_projected_grid_with_query(
                            &projected.grid,
                            &UiArrangedTree::default(),
                            UiHitTestQuery::new(point),
                        ),
                        hit_test_projected_grid_with_query(
                            &rebuilt.grid,
                            &UiArrangedTree::default(),
                            UiHitTestQuery::new(point),
                        )
                    );
                }
            }
        }
    }
}

#[test]
fn projected_geometry_batch_preflight_failure_preserves_grid() {
    let node_id = UiNodeId::new(1);
    let entry = hit_entry(node_id, node_id, 0, 0);
    let base_grid = test_hit_grid(vec![(entry.clone(), node_id)]);
    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(&base_grid, &[]);
    let before = projected.grid.clone();
    let mut invalid = entry;
    invalid.z_index = projected.overlay_z_base;
    let updated_base = UiHitTestIndex::from_grid(test_hit_grid(vec![(invalid, node_id)]));

    assert_eq!(
        projected.patch(&updated_base, &[], &BTreeSet::from([node_id])),
        Err(())
    );
    assert_eq!(projected.grid, before);
}

#[test]
fn projected_cold_lookup_preflight_failure_preserves_reverse_maps_and_grid() {
    let node_id = UiNodeId::new(1);
    let entry = hit_entry(node_id, node_id, 0, 0);
    let base_grid = test_hit_grid(vec![(entry.clone(), node_id)]);
    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(&base_grid, &[]);
    projected.entry_cells.clear();
    projected.entry_indices.clear();
    let before_grid = projected.grid.clone();
    let before_entry_cells = projected.entry_cells.clone();
    let before_entry_indices = projected.entry_indices.clone();

    let mut invalid = entry;
    invalid.z_index = projected.overlay_z_base;
    let updated_base = UiHitTestIndex::from_grid(test_hit_grid(vec![(invalid, node_id)]));

    assert_eq!(
        projected.patch(&updated_base, &[], &BTreeSet::from([node_id])),
        Err(())
    );
    assert_eq!(projected.grid, before_grid);
    assert_eq!(projected.entry_cells, before_entry_cells);
    assert_eq!(projected.entry_indices, before_entry_indices);
}

#[test]
fn projected_entry_insert_and_delete_fall_back_to_exact_rebuild() {
    let popup_root = UiNodeId::new(10);
    let first_id = UiNodeId::new(11);
    let second_id = UiNodeId::new(12);
    let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let mut first = hit_entry(first_id, popup_root, 0, 0);
    first.frame = frame;
    first.clip_frame = frame;
    let mut projected = UiProjectedHitTestIndex::default();
    let base_grid = test_hit_grid(vec![(first.clone(), popup_root)]);
    projected.rebuild(&base_grid, &[]);

    let mut second = hit_entry(second_id, popup_root, 0, 1);
    second.frame = frame;
    second.clip_frame = frame;
    let inserted_base = UiHitTestIndex::from_grid(test_hit_grid(vec![
        (first.clone(), popup_root),
        (second.clone(), popup_root),
    ]));
    assert!(projected.synchronize(&inserted_base, &[], &BTreeSet::from([second_id]), false,));
    assert_eq!(projected.grid.entries.len(), 2);
    assert_eq!(projected.grid.cells[0].entries.as_slice(), &[0, 1]);

    let deleted_base = UiHitTestIndex::from_grid(test_hit_grid(vec![(second, popup_root)]));
    assert!(projected.synchronize(&deleted_base, &[], &BTreeSet::from([first_id]), false,));
    assert_eq!(projected.grid.entries.len(), 1);
    assert_eq!(projected.grid.entries[0].node_id, second_id);
    assert_eq!(projected.grid.cells[0].entries.as_slice(), &[0]);
}

#[test]
fn projected_nonmonotonic_membership_move_matches_full_rebuild() {
    let first_id = UiNodeId::new(1);
    let second_id = UiNodeId::new(2);
    let anchor_id = UiNodeId::new(3);
    let source_frame = UiFrame::new(1.0, 1.0, 8.0, 8.0);
    let target_frame = UiFrame::new(80.0, 1.0, 8.0, 8.0);
    let mut first = hit_entry(first_id, first_id, 0, 0);
    first.frame = source_frame;
    first.clip_frame = source_frame;
    let mut second = hit_entry(second_id, second_id, 0, 1);
    second.frame = source_frame;
    second.clip_frame = source_frame;
    let mut anchor = hit_entry(anchor_id, anchor_id, 0, 2);
    anchor.frame = target_frame;
    anchor.clip_frame = target_frame;
    let base_grid = test_hit_grid(vec![
        (first.clone(), first_id),
        (second, second_id),
        (anchor, anchor_id),
    ]);
    let mut projected = UiProjectedHitTestIndex::default();
    projected.rebuild(&base_grid, &[]);

    projected.grid.entries[0].paint_order = 3;
    let source_cell = projected.entry_cells[&first_id][0];
    projected.grid.cells[source_cell]
        .entries
        .retain(|entry_index| *entry_index != 0);
    projected.grid.cells[source_cell].entries.push(0);
    assert_eq!(
        projected.grid.cells[source_cell].entries.as_slice(),
        &[1, 0]
    );
    first.paint_order = 3;
    first.frame = target_frame;
    first.clip_frame = target_frame;
    let updated_grid = test_hit_grid(vec![
        (first, first_id),
        (
            {
                let mut entry = hit_entry(second_id, second_id, 0, 1);
                entry.frame = source_frame;
                entry.clip_frame = source_frame;
                entry
            },
            second_id,
        ),
        (
            {
                let mut entry = hit_entry(anchor_id, anchor_id, 0, 2);
                entry.frame = target_frame;
                entry.clip_frame = target_frame;
                entry
            },
            anchor_id,
        ),
    ]);
    let updated_base = UiHitTestIndex::from_grid(updated_grid.clone());

    #[cfg(feature = "profiling")]
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    #[cfg(feature = "profiling")]
    {
        let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
        config.session_id = "ui04-projected-cell-membership-patch".to_owned();
        config.max_counters = 64;
        assert!(crate::core::diagnostics::profiling::start_capture(config).active);
    }
    assert!(projected
        .patch(&updated_base, &[], &BTreeSet::from([first_id]))
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
            let projected = format!("ui.surface_projected_hit.{suffix}");
            let base = format!("ui.hit_grid.{suffix}");
            assert!(
                profile.counters.iter().any(|counter| {
                    counter.stream == "runtime" && counter.name == projected && counter.value > 0.0
                }),
                "projected patch should record positive {projected}"
            );
            assert!(
                !profile
                    .counters
                    .iter()
                    .any(|counter| { counter.stream == "runtime" && counter.name == base }),
                "projected patch should not record base {base}"
            );
        }
    }
    assert_eq!(projected.grid.cells[source_cell].entries.as_slice(), &[1]);
    let mut rebuilt = UiProjectedHitTestIndex::default();
    rebuilt.rebuild(&updated_grid, &[]);
    for point in [UiPoint::new(4.0, 4.0), UiPoint::new(84.0, 4.0)] {
        let patched = hit_test_projected_grid_with_query(
            &projected.grid,
            &UiArrangedTree::default(),
            UiHitTestQuery::new(point),
        );
        let expected = hit_test_projected_grid_with_query(
            &rebuilt.grid,
            &UiArrangedTree::default(),
            UiHitTestQuery::new(point),
        );
        assert_eq!(patched.top_hit, expected.top_hit);
        assert_eq!(patched.stacked, expected.stacked);
        assert_eq!(patched.path, expected.path);
    }
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
