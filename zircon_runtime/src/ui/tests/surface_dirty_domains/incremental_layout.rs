use super::*;

#[test]
fn incremental_layout_records_geometry_deltas_during_arrangement() {
    let source = include_str!("../../layout/pass/incremental.rs");
    let arrange = include_str!("../../layout/pass/arrange.rs");

    assert!(!source.contains("BTreeMap"));
    assert!(!source.contains("snapshot_geometry"));
    assert!(!source.contains("collect_subtree_nodes"));
    assert!(arrange.contains("record_geometry"));
}

#[test]
fn incremental_layout_reports_only_current_taffy_tree_build_work() {
    let taffy_fixed = |size| AxisConstraint {
        min: 0.0,
        max: size,
        preferred: size,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    };
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.incremental_layout.taffy_current_pass",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(Default::default())),
    );
    for (node_id, path, width) in [
        (primary_id(), "root/primary", 40.0),
        (sibling_id(), "root/sibling", 40.0),
    ] {
        surface
            .tree
            .insert_child(
                root_id(),
                UiTreeNode::new(node_id, UiNodePath::new(path)).with_constraints(BoxConstraints {
                    width: taffy_fixed(width),
                    height: taffy_fixed(20.0),
                }),
            )
            .expect("Taffy fixture child should insert");
    }
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = taffy_fixed(48.0);
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.layout_taffy_tree_build_count, 1);
    assert_eq!(report.layout_taffy_tree_node_build_count, 3);
    assert_eq!(
        report.debug_stats().layout_taffy_tree_build_count,
        report.layout_taffy_tree_build_count
    );
    assert_eq!(
        report.debug_stats().layout_taffy_tree_node_build_count,
        report.layout_taffy_tree_node_build_count
    );

    let stable = surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(stable.layout_taffy_tree_build_count, 0);
    assert_eq!(stable.layout_taffy_tree_node_build_count, 0);
}

#[test]
fn deserialized_surface_rebuilds_geometry_for_the_first_new_root_size() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.incremental_layout.deserialize_resize",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root")).with_container(UiContainerKind::Free),
    );
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();

    let encoded = serde_json::to_string(&surface).expect("surface should serialize");
    let mut restored = serde_json::from_str::<UiSurface>(&encoded).expect("surface should restore");
    let resized_root = UiSize::new(240.0, 90.0);

    let report = restored.rebuild_dirty(resized_root).unwrap();

    assert!(report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert_eq!(
        restored
            .arranged_tree
            .get(root_id())
            .expect("restored root should be arranged")
            .frame,
        zircon_runtime_interface::ui::layout::UiFrame::new(0.0, 0.0, 240.0, 90.0)
    );
}

#[test]
fn root_resize_reuses_clean_descendant_measurement_and_arrangement() {
    let child_id = primary_id();
    let text_id = sibling_id();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.incremental_layout.resize_reuse"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root")).with_container(UiContainerKind::Free),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(child_id, UiNodePath::new("root/child"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                })
                .with_container(UiContainerKind::Free)
                .with_layout_boundary(LayoutBoundary::ParentDirected),
        )
        .expect("fixed child should insert");
    surface
        .tree
        .insert_child(
            child_id,
            UiTreeNode::new(text_id, UiNodePath::new("root/child/text"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(16.0),
                    height: fixed_constraint(12.0),
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Text".to_string(),
                    attributes: toml::from_str("text = \"retained resize text\"")
                        .expect("text metadata"),
                    ..UiTemplateNodeMetadata::default()
                })
                .with_layout_boundary(LayoutBoundary::ParentDirected),
        )
        .expect("fixed descendant should insert");
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    let child_frame = surface.tree.node(child_id).unwrap().layout_cache.frame;
    let text_frame = surface.tree.node(text_id).unwrap().layout_cache.frame;

    let report = surface
        .rebuild_dirty(UiSize::new(
            root_size().width + 80.0,
            root_size().height + 40.0,
        ))
        .unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_skipped_node_count, 2);
    assert_eq!(report.layout_geometry_changed_node_count, 1);
    assert_eq!(report.text_measure_cache_hit_count, 0);
    assert_eq!(report.text_measure_cache_miss_count, 0);
    assert_eq!(
        surface.tree.node(child_id).unwrap().layout_cache.frame,
        child_frame
    );
    assert_eq!(
        surface.tree.node(text_id).unwrap().layout_cache.frame,
        text_frame
    );
}

#[test]
fn root_resize_reports_early_out_probe_work() {
    const CHILD_COUNT: usize = 128;
    let mut surface = flat_scale_surface(CHILD_COUNT);

    let report = surface
        .rebuild_dirty(UiSize::new(
            root_size().width + 80.0,
            root_size().height + 40.0,
        ))
        .unwrap();

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_measure_probe_node_count, 0);
    assert_eq!(report.layout_arrange_probe_node_count, 1);
}

#[test]
fn root_resize_excludes_non_layout_dirty_nodes_from_the_layout_budget() {
    const CHILD_COUNT: usize = 128;
    let mut surface = flat_scale_surface(CHILD_COUNT);
    for child_index in 0..CHILD_COUNT {
        surface
            .invalidate_node(
                UiNodeId::new(child_index as u64 + 2),
                UiInvalidationReason::Render,
            )
            .expect("render-only child invalidation should succeed");
    }

    let report = surface
        .rebuild_dirty(UiSize::new(
            root_size().width + 80.0,
            root_size().height + 40.0,
        ))
        .unwrap();

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_measure_probe_node_count, 0);
    assert_eq!(report.layout_arrange_probe_node_count, 1);
}

#[test]
fn root_resize_combines_input_and_geometry_patches_without_full_rebuild() {
    const CHILD_COUNT: usize = 128;
    let child_id = UiNodeId::new(2);
    let mut surface = flat_scale_surface(CHILD_COUNT);
    surface
        .tree
        .node_mut(child_id)
        .expect("interaction child should exist")
        .constraints
        .width = AxisConstraint::default();
    surface
        .invalidate_node(child_id, UiInvalidationReason::Layout)
        .expect("stretched child invalidation should succeed");
    surface.rebuild_dirty(root_size()).unwrap();
    surface
        .tree
        .node_mut(child_id)
        .expect("interaction child should exist")
        .state_flags
        .enabled = false;
    surface
        .invalidate_node(child_id, UiInvalidationReason::Interaction)
        .expect("interaction child invalidation should succeed");

    let report = surface
        .rebuild_dirty(UiSize::new(
            root_size().width + 80.0,
            root_size().height + 40.0,
        ))
        .unwrap();

    assert_eq!(report.layout_visited_node_count, 2);
    assert_eq!(report.layout_measure_probe_node_count, 0);
    assert_eq!(report.layout_arrange_probe_node_count, 2);
    assert_eq!(report.arranged_outer_node_visit_count, 2);
    assert_eq!(report.hit_grid_outer_node_visit_count, 2);
    assert_eq!(report.render_outer_node_visit_count, 2);
    assert!(
        !surface
            .arranged_tree
            .get(child_id)
            .expect("interaction child should remain arranged")
            .enabled
    );
}

#[test]
fn root_resize_dependency_index_tracks_a_child_that_becomes_stretched() {
    let mut surface = flat_scale_surface(2);
    let stretched_id = UiNodeId::new(2);
    surface
        .tree
        .node_mut(stretched_id)
        .expect("stretched child should exist")
        .constraints
        .width = AxisConstraint::default();
    surface
        .invalidate_node(stretched_id, UiInvalidationReason::Layout)
        .unwrap();
    surface.rebuild_dirty(root_size()).unwrap();

    let resized = UiSize::new(root_size().width + 80.0, root_size().height + 40.0);
    let report = surface.rebuild_dirty(resized).unwrap();

    assert_eq!(report.layout_measure_probe_node_count, 0);
    assert_eq!(report.layout_arrange_probe_node_count, 2);
    assert_eq!(
        surface
            .tree
            .node(stretched_id)
            .expect("stretched child should remain arranged")
            .layout_cache
            .frame
            .width,
        resized.width
    );
}

#[test]
fn clipped_root_resize_uses_the_conservative_clip_propagation_path() {
    const CHILD_COUNT: usize = 4;
    let mut surface = flat_scale_surface(CHILD_COUNT);
    surface
        .tree
        .node_mut(root_id())
        .expect("root should exist")
        .clip_to_bounds = true;
    surface
        .invalidate_node(root_id(), UiInvalidationReason::Layout)
        .unwrap();
    surface.rebuild_dirty(root_size()).unwrap();

    let resized = UiSize::new(root_size().width + 80.0, root_size().height + 40.0);
    let report = surface.rebuild_dirty(resized).unwrap();

    assert_eq!(report.layout_measure_probe_node_count, 0);
    assert_eq!(report.layout_arrange_probe_node_count, CHILD_COUNT + 1);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .expect("child should remain arranged")
            .layout_cache
            .clip_frame,
        Some(UiFrame::new(0.0, 0.0, resized.width, resized.height))
    );
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
            .width = fixed_constraint(20.0);
        surface
            .invalidate_node(changed_node_id, UiInvalidationReason::Layout)
            .unwrap();

        let report = surface.rebuild_dirty(root_size()).unwrap();
        assert_eq!(report.layout_visited_node_count, 1, "{child_count}");
        assert_eq!(report.arranged_outer_node_visit_count, 1, "{child_count}");
        assert_eq!(report.hit_grid_outer_node_visit_count, 1, "{child_count}");
        assert_eq!(report.render_outer_node_visit_count, 1, "{child_count}");
        eprintln!(
            "nodes={} layout_visited={} arranged_outer_visited={} hit_outer_visited={} render_outer_visited={} layout_us={} arranged_us={} hit_us={} render_us={}",
            child_count + 1,
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
                    .with_state_flags(UiStateFlags {
                        visible: true,
                        enabled: true,
                        hoverable: true,
                        ..Default::default()
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
        .width = fixed_constraint(20.0);
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_skipped_node_count, 2);
    assert_eq!(report.layout_geometry_changed_node_count, 1);
    assert_eq!(
        surface.last_layout_geometry_changed_node_ids(),
        &std::collections::BTreeSet::from([primary_id()])
    );
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 1);
    assert_eq!(report.render_outer_node_visit_count, 1);
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
fn geometry_change_on_non_pointer_node_keeps_hit_grid_incremental() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .state_flags
        .clickable = false;
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .state_flags
        .hoverable = false;
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();
    surface.rebuild_dirty(root_size()).unwrap();
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(20.0);
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 0);
    assert!(!report.hit_grid_rebuilt);
}

#[test]
fn pointer_node_growing_from_zero_area_rebuilds_missing_hit_entry() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(0.0);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .state_flags = UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        ..Default::default()
    };
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();
    surface.rebuild_dirty(root_size()).unwrap();

    assert!(
        surface
            .hit_test
            .grid
            .entries
            .iter()
            .all(|entry| entry.node_id != primary_id())
    );

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(60.0);
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(
        report.hit_grid_outer_node_visit_count,
        surface.arranged_tree.draw_order.len()
    );
    assert!(report.hit_grid_rebuilt);
    assert!(
        surface
            .hit_test
            .grid
            .entries
            .iter()
            .any(|entry| entry.node_id == primary_id())
    );
}

#[test]
fn mixed_layout_and_input_dirty_rebuilds_arranged_and_hit_state() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(60.0);
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Layout)
        .unwrap();
    surface
        .tree
        .node_mut(sibling_id())
        .expect("sibling node should exist")
        .state_flags
        .clickable = false;
    surface
        .tree
        .node_mut(sibling_id())
        .expect("sibling node should exist")
        .state_flags
        .hoverable = false;
    surface
        .mark_node_dirty(
            sibling_id(),
            UiDirtyFlags {
                hit_test: true,
                render: true,
                input: true,
                ..Default::default()
            },
        )
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(
        report.arranged_outer_node_visit_count,
        surface.tree.nodes.len()
    );
    assert_eq!(
        report.hit_grid_outer_node_visit_count,
        surface.arranged_tree.draw_order.len()
    );
    assert!(report.hit_grid_rebuilt);
    assert!(
        surface
            .hit_test
            .grid
            .entries
            .iter()
            .all(|entry| entry.node_id != sibling_id())
    );
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
    assert!(
        !surface
            .layout_engine_report
            .selections
            .iter()
            .any(|selection| selection.node_id == Some(primary_id())
                && selection.request.family == UiLayoutEngineFamily::Flex)
    );
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
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_skipped_node_count, 1);
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
fn measured_but_geometry_reused_container_preserves_layout_engine_route() {
    let zero_viewport = UiSize::new(0.0, 0.0);
    let mut surface = layout_route_merge_surface();
    surface.compute_layout(zero_viewport).unwrap();
    surface.clear_dirty_flags();
    let initial_report = surface.layout_engine_report.clone();

    assert_eq!(route_count_for_node(&initial_report, primary_id()), 1);
    surface
        .invalidate_node(root_id(), UiInvalidationReason::Layout)
        .unwrap();

    let report = surface.rebuild_dirty(zero_viewport).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_geometry_changed_node_count, 0);
    assert_eq!(surface.layout_engine_report, initial_report);
    assert_eq!(
        route_count_for_node(&surface.layout_engine_report, primary_id()),
        1
    );
    assert_layout_engine_report_exported(&surface, &initial_report);
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
        surface.last_layout_geometry_changed_node_ids(),
        &std::collections::BTreeSet::from([primary_id(), sibling_id()])
    );
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
