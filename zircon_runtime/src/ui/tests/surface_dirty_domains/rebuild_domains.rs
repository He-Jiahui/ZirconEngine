use super::*;

#[test]
fn surface_dirty_rebuild_separates_hit_input_render_and_legacy_state_flags() {
    let mut surface = test_surface();

    mark_structured_dirty(
        &mut surface,
        UiDirtyFlags {
            hit_test: true,
            ..Default::default()
        },
    );
    let hit_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        hit_report,
        UiDirtyFlags {
            hit_test: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: true,
            hit_grid: true,
            render: false,
        },
    );
    assert_dirty_cleared(&surface);

    mark_structured_dirty(
        &mut surface,
        UiDirtyFlags {
            input: true,
            ..Default::default()
        },
    );
    let input_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        input_report,
        UiDirtyFlags {
            input: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: true,
            hit_grid: true,
            render: false,
        },
    );
    assert_dirty_cleared(&surface);

    mark_structured_dirty(
        &mut surface,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
    );
    let render_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        render_report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);

    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .state_flags
        .dirty = true;
    let legacy_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        legacy_report,
        UiDirtyFlags {
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: true,
            hit_grid: true,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_rebuild_recomputes_layout_for_structural_domains() {
    for dirty_flags in [
        UiDirtyFlags {
            layout: true,
            ..Default::default()
        },
        UiDirtyFlags {
            style: true,
            ..Default::default()
        },
        UiDirtyFlags {
            text: true,
            ..Default::default()
        },
        UiDirtyFlags {
            visible_range: true,
            ..Default::default()
        },
    ] {
        let mut surface = test_surface();

        mark_structured_dirty(&mut surface, dirty_flags);
        let report = surface.rebuild_dirty(root_size()).unwrap();

        assert_report_phases(
            &surface,
            report,
            dirty_flags,
            ExpectedPhases {
                layout: true,
                arranged: true,
                hit_grid: dirty_flags.style || dirty_flags.visible_range,
                render: true,
            },
        );
        assert_dirty_cleared(&surface);
    }
}

#[test]
fn interaction_dirty_patches_only_the_changed_button() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);

    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Interaction)
        .unwrap();
    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(!report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert!(report.hit_grid_rebuilt);
    assert!(report.render_rebuilt);
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 1);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert!(surface
        .hit_test
        .grid
        .entries
        .iter()
        .any(|entry| entry.node_id == primary_id()));
}

#[test]
fn pointer_event_eligibility_change_falls_back_and_removes_the_hit_entry() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .pointer_events = zircon_runtime_interface::ui::tree::UiPointerEvents::None;
    surface
        .mark_node_dirty(
            primary_id(),
            UiDirtyFlags {
                hit_test: true,
                input: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(
        report.hit_grid_outer_node_visit_count,
        surface.arranged_tree.draw_order.len()
    );
    assert!(surface
        .hit_test
        .grid
        .entries
        .iter()
        .all(|entry| entry.node_id != primary_id()));
    assert!(surface
        .hit_test
        .grid
        .entries
        .iter()
        .any(|entry| entry.node_id == sibling_id()));
}
