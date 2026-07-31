use super::*;

#[test]
fn incremental_layout_snapshots_only_visited_geometry() {
    let source = include_str!("../../layout/pass/incremental.rs");

    assert!(source.contains("let previous = snapshot_geometry(tree, &visited);"));
    assert!(source.contains("visited: &BTreeSet<UiNodeId>"));
    assert!(!source.contains("let previous = snapshot_geometry(tree);"));
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
