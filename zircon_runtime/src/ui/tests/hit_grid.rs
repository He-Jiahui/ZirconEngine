use crate::ui::{
    surface::{hit_test_surface_frame, UiSurface},
    tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeScrollExt},
};
use zircon_runtime_interface::ui::surface::{UiHitTestQuery, UiVirtualPointerPosition};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn hit_grid_omits_disabled_nodes_and_debug_dump_reports_why() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/enabled"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/disabled"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0))
                .with_z_index(20)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(disabled_pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("disabled.button".to_string()),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 20.0)).top_hit,
        Some(UiNodeId::new(2))
    );
    assert!(surface
        .hit_test
        .grid
        .entries
        .iter()
        .all(|entry| entry.node_id != UiNodeId::new(3)));

    let dump = surface.debug_hit_test(UiPoint::new(20.0, 20.0));
    assert!(dump
        .rejected
        .iter()
        .any(|reject| reject.node_id == UiNodeId::new(3)
            && reject.control_id.as_deref() == Some("disabled.button")
            && reject.reason == "disabled"));
}

#[test]
fn scrollable_virtualized_children_enter_hit_grid_only_when_arranged_visible() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: fixed_constraint(200.0),
                height: fixed_constraint(80.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(200.0),
                    height: fixed_constraint(80.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 0,
                    }),
                }))
                .with_scroll_state(UiScrollState::default())
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();

    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(200.0),
                    height: fixed_constraint(40.0),
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();

    assert!(hit_grid_contains(&surface, UiNodeId::new(10)));
    assert!(hit_grid_contains(&surface, UiNodeId::new(11)));
    assert!(!hit_grid_contains(&surface, UiNodeId::new(12)));
    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 20.0)).top_hit,
        Some(UiNodeId::new(10))
    );

    surface
        .tree
        .set_scroll_offset(UiNodeId::new(2), 80.0)
        .unwrap();
    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();

    assert!(!hit_grid_contains(&surface, UiNodeId::new(10)));
    assert!(!hit_grid_contains(&surface, UiNodeId::new(11)));
    assert!(hit_grid_contains(&surface, UiNodeId::new(12)));
    assert!(hit_grid_contains(&surface, UiNodeId::new(13)));
    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 20.0)).top_hit,
        Some(UiNodeId::new(12))
    );
}

#[test]
fn surface_frame_carries_focus_capture_and_hover_snapshot() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 5.0, 60.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();
    surface.focus.focused = Some(UiNodeId::new(2));
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.focus.hovered = vec![UiNodeId::new(2), UiNodeId::new(1)];

    let frame = surface.surface_frame();

    assert_eq!(frame.focus_state.focused, Some(UiNodeId::new(2)));
    assert_eq!(frame.focus_state.captured, Some(UiNodeId::new(2)));
    assert_eq!(
        frame.focus_state.hovered,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn hit_grid_uses_cursor_radius_as_slate_style_nearby_hit_fallback() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/left"))
                .with_frame(UiFrame::new(20.0, 20.0, 20.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/right"))
                .with_frame(UiFrame::new(52.0, 20.0, 20.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();

    assert_eq!(surface.hit_test(UiPoint::new(46.0, 30.0)).top_hit, None);

    let hit = surface
        .hit_test_with_query(UiHitTestQuery::new(UiPoint::new(46.0, 30.0)).with_cursor_radius(8.0));

    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(
        hit.path.root_to_leaf,
        vec![UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(hit.stacked, vec![UiNodeId::new(3), UiNodeId::new(2)]);
}

#[test]
fn exact_hit_wins_over_nearby_cursor_radius_candidates() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/exact"))
                .with_frame(UiFrame::new(40.0, 20.0, 20.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/front_nearby"))
                .with_frame(UiFrame::new(62.0, 20.0, 20.0, 20.0))
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();

    let hit = surface.hit_test_with_query(
        UiHitTestQuery::new(UiPoint::new(50.0, 30.0)).with_cursor_radius(16.0),
    );

    assert_eq!(hit.top_hit, Some(UiNodeId::new(2)));
    assert_eq!(hit.stacked, vec![UiNodeId::new(2), UiNodeId::new(3)]);
}

#[test]
fn virtual_pointer_query_maps_custom_3d_hits_into_surface_local_hit_path() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/world_space_button"))
                .with_frame(UiFrame::new(80.0, 40.0, 30.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();

    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(90.0, 50.0), UiPoint::new(85.0, 45.0));
    let hit = surface.hit_test_with_query(
        UiHitTestQuery::new(UiPoint::new(5.0, 5.0)).with_virtual_pointer(virtual_pointer),
    );

    assert_eq!(hit.top_hit, Some(UiNodeId::new(2)));
    assert_eq!(hit.path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(
        hit.path.bubble_route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn surface_frame_hit_test_uses_borrowed_grid_with_index_parity() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 90.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/back"))
                .with_frame(UiFrame::new(12.0, 12.0, 80.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/front"))
                .with_frame(UiFrame::new(32.0, 20.0, 80.0, 40.0))
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();
    let frame = surface.surface_frame();
    let point = UiPoint::new(40.0, 24.0);

    let index_hit = surface.hit_test(point);
    let frame_hit = hit_test_surface_frame(&frame, point);

    assert_eq!(frame_hit, index_hit);
    assert_eq!(frame_hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(frame_hit.stacked, vec![UiNodeId::new(3), UiNodeId::new(2)]);
}

#[test]
fn surface_dirty_rebuild_keeps_render_only_changes_out_of_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: fixed_constraint(100.0),
                height: fixed_constraint(40.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(80.0),
                    height: fixed_constraint(20.0),
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(100.0, 40.0)).unwrap();
    let before_frame = surface
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("button should be arranged")
        .frame;

    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("button node should exist")
        .dirty
        .render = true;
    let render_report = surface.rebuild_dirty(UiSize::new(100.0, 40.0)).unwrap();

    assert!(!render_report.layout_recomputed);
    assert!(!render_report.arranged_rebuilt);
    assert!(!render_report.hit_grid_rebuilt);
    assert!(render_report.render_rebuilt);
    assert!(render_report.dirty_flags.render);
    assert_eq!(render_report.dirty_node_count, 1);
    assert_eq!(render_report.arranged_node_count, 2);
    assert_eq!(render_report.render_command_count, 2);
    assert_eq!(render_report.hit_grid_entry_count, surface.hit_test.grid.entries.len());
    assert_eq!(render_report.hit_grid_cell_count, surface.hit_test.grid.cells.len());
    assert_eq!(surface.surface_frame().last_rebuild, render_report.debug_stats());
    assert_eq!(
        surface
            .arranged_tree
            .get(UiNodeId::new(2))
            .expect("button should stay arranged")
            .frame,
        before_frame
    );
    assert!(!surface.dirty_flags().any());

    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("button node should exist")
        .dirty
        .layout = true;
    let layout_report = surface.rebuild_dirty(UiSize::new(100.0, 40.0)).unwrap();

    assert!(layout_report.layout_recomputed);
    assert!(layout_report.arranged_rebuilt);
    assert!(layout_report.hit_grid_rebuilt);
    assert!(layout_report.render_rebuilt);
    assert!(layout_report.dirty_flags.layout);
    assert_eq!(layout_report.dirty_node_count, 1);
    assert_eq!(layout_report.arranged_node_count, 2);
    assert_eq!(layout_report.render_command_count, 2);
    assert_eq!(layout_report.hit_grid_entry_count, surface.hit_test.grid.entries.len());
    assert_eq!(layout_report.hit_grid_cell_count, surface.hit_test.grid.cells.len());
    assert_eq!(surface.surface_frame().last_rebuild, layout_report.debug_stats());
    assert!(!surface.dirty_flags().any());
}

#[test]
fn surface_dirty_rebuild_records_cached_counts_when_no_dirty_flags_exist() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();
    surface.clear_dirty_flags();

    let report = surface.rebuild_dirty(UiSize::new(120.0, 60.0)).unwrap();

    assert!(!report.dirty_flags.any());
    assert_eq!(report.dirty_node_count, 0);
    assert_eq!(report.arranged_node_count, 2);
    assert_eq!(report.render_command_count, 2);
    assert_eq!(report.hit_grid_entry_count, surface.hit_test.grid.entries.len());
    assert_eq!(report.hit_grid_cell_count, surface.hit_test.grid.cells.len());
    assert_eq!(surface.surface_frame().last_rebuild, report.debug_stats());
}

fn hit_grid_contains(surface: &UiSurface, node_id: UiNodeId) -> bool {
    surface
        .hit_test
        .grid
        .entries
        .iter()
        .any(|entry| entry.node_id == node_id)
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn disabled_pointer_state() -> UiStateFlags {
    UiStateFlags {
        enabled: false,
        ..pointer_state()
    }
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
