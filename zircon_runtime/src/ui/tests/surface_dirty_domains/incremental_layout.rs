use super::*;

#[test]
fn incremental_layout_snapshots_only_visited_geometry() {
    let source = include_str!("../../layout/pass/incremental.rs");

    assert!(source.contains("let previous = snapshot_geometry(tree, &visited);"));
    assert!(source.contains("visited: &BTreeSet<UiNodeId>"));
    assert!(!source.contains("let previous = snapshot_geometry(tree);"));
}

#[test]
#[ignore = "explicit M0 scale matrix; run at milestone performance gates"]
fn stable_and_single_node_dirty_scale_matrix_exposes_post_layout_outer_traversals() {
    for child_count in [1_usize, 100, 10_000] {
        let mut surface = flat_scale_surface(child_count);
        let stable = surface.rebuild_dirty(root_size()).unwrap();
        assert!(!stable.dirty_flags.any(), "{child_count}");
        assert!(!stable.layout_recomputed, "{child_count}");
        assert!(!stable.arranged_rebuilt, "{child_count}");
        assert!(!stable.hit_grid_rebuilt, "{child_count}");
        assert!(!stable.render_rebuilt, "{child_count}");
        assert_eq!(stable.layout_visited_node_count, 0, "{child_count}");
        assert_eq!(stable.arranged_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.hit_grid_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.render_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.text_measure_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_measure_cache_miss_count, 0, "{child_count}");
        assert_eq!(stable.text_layout_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_layout_cache_miss_count, 0, "{child_count}");
        assert_eq!(stable.text_shape_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_shape_cache_miss_count, 0, "{child_count}");

        let changed_node_id = UiNodeId::new(child_count as u64 + 1);
        surface
            .tree
            .node_mut(changed_node_id)
            .expect("scale-matrix node should exist")
            .constraints
            .width = fixed_constraint(60.0);
        surface
            .tree
            .node_mut(changed_node_id)
            .expect("scale-matrix node should exist")
            .dirty
            .layout = true;

        let report = surface.rebuild_dirty(root_size()).unwrap();
        let total_node_count = child_count + 1;

        assert_eq!(report.layout_visited_node_count, 1, "{child_count}");
        assert_eq!(
            report.arranged_outer_node_visit_count, total_node_count,
            "{child_count}"
        );
        assert_eq!(
            report.hit_grid_outer_node_visit_count, total_node_count,
            "{child_count}"
        );
        assert_eq!(
            report.render_outer_node_visit_count, total_node_count,
            "{child_count}"
        );
        eprintln!(
            "nodes={total_node_count} layout_visited={} arranged_outer_visited={} hit_outer_visited={} render_outer_visited={} layout_us={} arranged_us={} hit_us={} render_us={}",
            report.layout_visited_node_count,
            report.arranged_outer_node_visit_count,
            report.hit_grid_outer_node_visit_count,
            report.render_outer_node_visit_count,
            report.layout_elapsed_micros,
            report.arranged_elapsed_micros,
            report.hit_grid_elapsed_micros,
            report.render_elapsed_micros,
        );
    }
}

fn flat_scale_surface(child_count: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.incremental_layout.scale.{child_count}"
    )));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            })
            .with_container(UiContainerKind::Free)
            .with_layout_boundary(LayoutBoundary::ParentDirected),
    );
    for child_index in 0..child_count {
        let node_id = UiNodeId::new(child_index as u64 + 2);
        surface
            .tree
            .insert_child(
                root_id(),
                UiTreeNode::new(node_id, UiNodePath::new(format!("root/{child_index}")))
                    .with_constraints(BoxConstraints {
                        width: fixed_constraint(40.0),
                        height: fixed_constraint(20.0),
                    })
                    .with_layout_boundary(LayoutBoundary::ParentDirected),
            )
            .expect("scale-matrix node should insert");
    }
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

#[test]
fn surface_dirty_layout_skips_siblings_under_non_auto_parent() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    let sibling_frame = surface
        .arranged_tree
        .get(sibling_id())
        .expect("sibling should be arranged")
        .frame;

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(60.0);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .dirty
        .layout = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_skipped_node_count, 2);
    assert_eq!(report.layout_geometry_changed_node_count, 1);
    assert_eq!(
        report.arranged_outer_node_visit_count,
        surface.tree.nodes.len()
    );
    assert_eq!(
        report.hit_grid_outer_node_visit_count,
        surface.arranged_tree.draw_order.len()
    );
    assert_eq!(
        report.render_outer_node_visit_count,
        surface.arranged_tree.draw_order.len()
    );
    assert_eq!(report.render_command_rebuilt_count, 1);
    assert_eq!(report.render_damage_rect_count, 1);
    assert_eq!(
        surface
            .arranged_tree
            .get(sibling_id())
            .expect("sibling should stay arranged")
            .frame,
        sibling_frame
    );
    assert_dirty_cleared_for(&surface, primary_id());
}

#[test]
fn surface_dirty_layout_preserves_unvisited_layout_engine_routes() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    let initial_report = surface.layout_engine_report.clone();
    let root_selection = initial_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(root_id()))
        .expect("root route should be reported");

    assert_eq!(initial_report.request_count, 1);
    assert_fallback_reason_count(
        &initial_report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        1,
    );
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Free);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Zircon
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(60.0);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .dirty
        .layout = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_skipped_node_count, 2);
    assert_eq!(surface.layout_engine_report, initial_report);
    assert_fallback_reason_count(
        &surface.layout_engine_report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        1,
    );
    assert_layout_engine_report_exported(&surface, &initial_report);
    assert_dirty_cleared_for(&surface, primary_id());
}

#[test]
fn surface_dirty_layout_replaces_visited_layout_engine_routes() {
    let mut surface = layout_route_merge_surface();
    let initial_report = surface.layout_engine_report.clone();

    assert_eq!(initial_report.request_count, 2);
    assert_fallback_reason_count(
        &initial_report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        1,
    );
    assert_fallback_reason_count(
        &initial_report,
        UiLayoutEngineFallbackReason::AxisConstraintPriority,
        1,
    );
    assert_route(
        &initial_report,
        root_id(),
        UiLayoutEngineFamily::Free,
        UiLayoutEngineBackend::Zircon,
        UiLayoutEngineSupport::Fallback,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics),
    );
    assert_route(
        &initial_report,
        primary_id(),
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineBackend::Zircon,
        UiLayoutEngineSupport::Fallback,
        Some(UiLayoutEngineFallbackReason::AxisConstraintPriority),
    );

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary route node should exist")
        .container = UiContainerKind::Overlay;
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary route node should exist")
        .dirty
        .layout = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 2);
    assert_eq!(report.layout_skipped_node_count, 2);
    assert_route(
        &surface.layout_engine_report,
        root_id(),
        UiLayoutEngineFamily::Free,
        UiLayoutEngineBackend::Zircon,
        UiLayoutEngineSupport::Fallback,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics),
    );
    assert_route(
        &surface.layout_engine_report,
        primary_id(),
        UiLayoutEngineFamily::Overlay,
        UiLayoutEngineBackend::Zircon,
        UiLayoutEngineSupport::Fallback,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics),
    );
    assert_eq!(
        route_count_for_node(&surface.layout_engine_report, primary_id()),
        1
    );
    assert_fallback_reason_count(
        &surface.layout_engine_report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        2,
    );
    assert!(!surface
        .layout_engine_report
        .selections
        .iter()
        .any(|selection| selection.node_id == Some(primary_id())
            && selection.request.family == UiLayoutEngineFamily::Flex));
    assert_layout_engine_report_exported(&surface, &surface.layout_engine_report);
}

#[test]
fn surface_dirty_layout_drops_removed_layout_engine_routes() {
    let mut surface = layout_route_merge_surface();

    assert_route(
        &surface.layout_engine_report,
        primary_id(),
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineBackend::Zircon,
        UiLayoutEngineSupport::Fallback,
        Some(UiLayoutEngineFallbackReason::AxisConstraintPriority),
    );

    surface.detach_subtree_to_pool(primary_id()).unwrap();
    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 2);
    assert_eq!(report.layout_skipped_node_count, 0);
    assert!(!surface.tree.nodes.contains_key(&primary_id()));
    assert_eq!(surface.layout_engine_report.request_count, 1);
    assert_fallback_reason_count(
        &surface.layout_engine_report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        1,
    );
    assert_route(
        &surface.layout_engine_report,
        root_id(),
        UiLayoutEngineFamily::Free,
        UiLayoutEngineBackend::Zircon,
        UiLayoutEngineSupport::Fallback,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics),
    );
    assert_eq!(
        route_count_for_node(&surface.layout_engine_report, primary_id()),
        0
    );
    assert_layout_engine_report_exported(&surface, &surface.layout_engine_report);
}

#[test]
fn surface_dirty_layout_revisits_auto_parent_when_child_size_changes() {
    let mut surface = sibling_surface(
        UiContainerKind::VerticalBox(Default::default()),
        LayoutBoundary::ParentDirected,
    );

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .height = fixed_constraint(40.0);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .dirty
        .layout = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 3);
    assert_eq!(report.layout_skipped_node_count, 0);
    assert_eq!(report.layout_geometry_changed_node_count, 2);
    assert_eq!(
        surface
            .arranged_tree
            .get(sibling_id())
            .expect("auto-layout sibling should be rearranged")
            .frame
            .y,
        40.0
    );
    assert_dirty_cleared_for(&surface, primary_id());
}
