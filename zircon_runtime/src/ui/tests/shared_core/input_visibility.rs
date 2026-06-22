use super::*;

#[test]
fn pointer_dispatcher_exposes_pointer_button_to_shared_route_handlers() {
    use std::sync::{Arc, Mutex};

    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface.rebuild();

    let seen_buttons = Arc::new(Mutex::new(Vec::new()));
    let seen_buttons_for_handler = Arc::clone(&seen_buttons);
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(1), UiPointerEventKind::Down, move |context| {
        seen_buttons_for_handler
            .lock()
            .unwrap()
            .push(context.route.button);
        UiPointerDispatchEffect::capture()
    });

    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
                .with_button(UiPointerButton::Secondary),
        )
        .unwrap();

    assert_eq!(result.route.button, Some(UiPointerButton::Secondary));
    assert_eq!(
        seen_buttons.lock().unwrap().as_slice(),
        &[Some(UiPointerButton::Secondary)]
    );
}

#[test]
fn hit_testing_respects_z_order_input_policy_and_clip_chain() {
    let mut tree = UiTree::new(UiTreeId::new("runtime.ui"));
    tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/background"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
            .with_z_index(0)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/overlay_ignore"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
            .with_z_index(100)
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/clipped_parent"))
            .with_frame(UiFrame::new(0.0, 0.0, 60.0, 60.0))
            .with_clip_to_bounds(true)
            .with_z_index(10)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(4),
        UiTreeNode::new(
            UiNodeId::new(5),
            UiNodePath::new("root/clipped_parent/child"),
        )
        .with_frame(UiFrame::new(20.0, 20.0, 100.0, 100.0))
        .with_z_index(30)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: false,
            pressed: false,
            checked: false,
            dirty: false,
        }),
    )
    .unwrap();

    let mut hit_test = UiHitTestIndex::default();
    hit_test.rebuild(&tree);

    let clipped = hit_test.hit_test(&tree, UiPoint::new(80.0, 80.0));
    assert_eq!(clipped.top_hit, Some(UiNodeId::new(2)));
    assert_eq!(clipped.stacked, vec![UiNodeId::new(2), UiNodeId::new(1)]);

    let unclipped = hit_test.hit_test(&tree, UiPoint::new(40.0, 40.0));
    assert_eq!(unclipped.top_hit, Some(UiNodeId::new(5)));
    assert_eq!(
        unclipped.stacked,
        vec![
            UiNodeId::new(5),
            UiNodeId::new(4),
            UiNodeId::new(2),
            UiNodeId::new(1)
        ]
    );
}

#[test]
fn surface_rebuild_derives_render_and_hit_from_same_arranged_geometry() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    let root_frame = UiFrame::new(0.0, 0.0, 160.0, 80.0);
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 5.0, 80.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("primary".to_string()),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let arranged = surface.arranged_tree.get(UiNodeId::new(2)).unwrap();
    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    let hit_entry = surface
        .hit_test
        .grid
        .entries
        .iter()
        .find(|entry| entry.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(arranged.frame, UiFrame::new(10.0, 5.0, 80.0, 20.0));
    assert_eq!(command.frame, arranged.frame);
    assert_eq!(hit_entry.frame, arranged.frame);
    assert_eq!(
        surface.hit_test(UiPoint::new(50.0, 15.0)).top_hit,
        Some(UiNodeId::new(2))
    );

    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(30.0, 25.0, 90.0, 24.0);
    surface.rebuild();

    let moved = surface.arranged_tree.get(UiNodeId::new(2)).unwrap();
    let moved_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    let moved_hit_entry = surface
        .hit_test
        .grid
        .entries
        .iter()
        .find(|entry| entry.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(moved.frame, UiFrame::new(30.0, 25.0, 90.0, 24.0));
    assert_eq!(moved_command.frame, moved.frame);
    assert_eq!(moved_hit_entry.frame, moved.frame);
    assert_eq!(surface.hit_test(UiPoint::new(50.0, 15.0)).top_hit, None);
    assert_eq!(
        surface.hit_test(UiPoint::new(50.0, 35.0)).top_hit,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn hit_grid_respects_slate_visibility_and_clip_semantics() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    let root_frame = UiFrame::new(0.0, 0.0, 120.0, 120.0);
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for (node_id, path, frame, z, visibility) in [
        (
            UiNodeId::new(2),
            "root/base",
            UiFrame::new(10.0, 10.0, 30.0, 20.0),
            0,
            UiVisibility::Visible,
        ),
        (
            UiNodeId::new(3),
            "root/hidden",
            UiFrame::new(10.0, 10.0, 30.0, 20.0),
            20,
            UiVisibility::Hidden,
        ),
        (
            UiNodeId::new(4),
            "root/collapsed",
            UiFrame::new(10.0, 10.0, 30.0, 20.0),
            30,
            UiVisibility::Collapsed,
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_z_index(z)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_visibility(visibility)
                    .with_state_flags(pointer_state()),
            )
            .unwrap();
    }

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(5), UiNodePath::new("root/hit_test_invisible"))
                .with_frame(UiFrame::new(60.0, 10.0, 40.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_visibility(UiVisibility::HitTestInvisible)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(5),
            UiTreeNode::new(
                UiNodeId::new(6),
                UiNodePath::new("root/hit_test_invisible/child"),
            )
            .with_frame(UiFrame::new(65.0, 15.0, 25.0, 20.0))
            .with_z_index(50)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(7),
                UiNodePath::new("root/self_hit_test_invisible"),
            )
            .with_frame(UiFrame::new(50.0, 60.0, 50.0, 40.0))
            .with_input_policy(UiInputPolicy::Receive)
            .with_visibility(UiVisibility::SelfHitTestInvisible)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(7),
            UiTreeNode::new(
                UiNodeId::new(8),
                UiNodePath::new("root/self_hit_test_invisible/child"),
            )
            .with_frame(UiFrame::new(55.0, 65.0, 30.0, 20.0))
            .with_z_index(60)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(pointer_state()),
        )
        .unwrap();

    surface.rebuild();

    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 18.0)).top_hit,
        Some(UiNodeId::new(2)),
        "hidden and collapsed overlap should not displace the visible base button"
    );
    assert_eq!(
        surface.hit_test(UiPoint::new(70.0, 22.0)).top_hit,
        None,
        "HitTestInvisible should block its own subtree from hit testing"
    );
    assert_eq!(
        surface.hit_test(UiPoint::new(70.0, 72.0)).top_hit,
        Some(UiNodeId::new(8)),
        "SelfHitTestInvisible should skip the parent but preserve child hits"
    );
    assert!(surface
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == UiNodeId::new(5)));
    assert!(!surface
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == UiNodeId::new(3)));
}

#[test]
fn legacy_visible_false_is_normalized_into_hidden_visibility_for_surface_outputs() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 80.0))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    let mut hidden_state = pointer_state();
    hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/legacy_hidden"))
                .with_frame(UiFrame::new(10.0, 10.0, 60.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(hidden_state),
        )
        .unwrap();

    surface.rebuild();

    let arranged = surface.arranged_tree.get(UiNodeId::new(2)).unwrap();
    assert_eq!(arranged.visibility, UiVisibility::Hidden);
    assert!(surface
        .render_extract
        .list
        .commands
        .iter()
        .all(|command| command.node_id != UiNodeId::new(2)));
    assert!(surface
        .hit_test
        .grid
        .entries
        .iter()
        .all(|entry| entry.node_id != UiNodeId::new(2)));
    assert_eq!(surface.hit_test(UiPoint::new(20.0, 15.0)).top_hit, None);
}

#[test]
fn explicit_collapsed_visibility_preserves_layout_collapse_with_legacy_visible_false() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(100.0),
                height: fixed_constraint(100.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/top")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(100.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    let mut legacy_hidden_state = pointer_state();
    legacy_hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/collapsed"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(100.0),
                    height: fixed_constraint(20.0),
                })
                .with_visibility(UiVisibility::Collapsed)
                .with_state_flags(legacy_hidden_state),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/bottom")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(100.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();

    let collapsed = surface.arranged_tree.get(UiNodeId::new(3)).unwrap();
    assert_eq!(collapsed.visibility, UiVisibility::Collapsed);
    assert_eq!(collapsed.frame, UiFrame::default());
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .expect("bottom node should be arranged")
            .layout_cache
            .frame
            .y,
        20.0
    );
}

#[test]
fn taffy_vertical_layout_skips_collapsed_child_without_fallback() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.taffy.collapsed"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_constraints(BoxConstraints {
                width: taffy_fixed_constraint(100.0),
                height: taffy_fixed_constraint(100.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/top")).with_constraints(
                BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/collapsed"))
                .with_visibility(UiVisibility::Collapsed)
                .with_constraints(BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(3),
            UiTreeNode::new(UiNodeId::new(30), UiNodePath::new("root/collapsed/child"))
                .with_constraints(BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/bottom")).with_constraints(
                BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 100.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(30))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 20.0, 100.0, 20.0)
    );

    let root_selection = surface
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("root should record a layout engine selection");
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(surface.layout_engine_report.fallback_count, 0);
    assert_eq!(surface.layout_engine_report.taffy_tree_node_count, 3);
}

#[test]
fn focus_navigation_and_scroll_candidates_use_effective_visibility() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/visible_focus"))
                .with_frame(UiFrame::new(0.0, 0.0, 40.0, 20.0))
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(3),
                UiNodePath::new("root/hit_test_invisible_focus"),
            )
            .with_frame(UiFrame::new(50.0, 0.0, 40.0, 20.0))
            .with_visibility(UiVisibility::HitTestInvisible)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    let mut legacy_hidden_state = pointer_state();
    legacy_hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(4),
                UiNodePath::new("root/legacy_hidden_focus"),
            )
            .with_frame(UiFrame::new(100.0, 0.0, 40.0, 20.0))
            .with_state_flags(legacy_hidden_state),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(5), UiNodePath::new("root/scroll"))
                .with_frame(UiFrame::new(0.0, 40.0, 100.0, 50.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState::default())
                .with_visibility(UiVisibility::SelfHitTestInvisible)
                .with_state_flags({
                    let mut state = pointer_state();
                    state.focusable = false;
                    state
                }),
        )
        .unwrap();

    assert_eq!(surface.focus_node(UiNodeId::new(3)), Ok(()));
    assert!(surface.focus_node(UiNodeId::new(4)).is_err());
    let focus_order = surface.tree.focusable_nodes_in_navigation_order().unwrap();
    assert!(focus_order.contains(&UiNodeId::new(2)));
    assert!(focus_order.contains(&UiNodeId::new(3)));
    assert!(!focus_order.contains(&UiNodeId::new(4)));
    assert_eq!(
        surface
            .tree
            .first_scrollable_in_candidates(&[UiNodeId::new(5)])
            .unwrap(),
        Some(UiNodeId::new(5))
    );
}

#[test]
fn pointer_capture_routes_move_and_up_to_the_captured_node() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/left"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 120.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: false,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/right"))
                .with_frame(UiFrame::new(120.0, 0.0, 100.0, 120.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface.rebuild();

    let down = surface
        .route_pointer_event(UiPointerEventKind::Down, UiPoint::new(130.0, 20.0))
        .unwrap();
    assert_eq!(down.target, Some(UiNodeId::new(3)));
    assert_eq!(down.bubbled, vec![UiNodeId::new(3), UiNodeId::new(1)]);
    assert_eq!(down.hit_path.target, Some(UiNodeId::new(3)));
    assert_eq!(
        down.hit_path.root_to_leaf,
        vec![UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(
        down.hit_path.bubble_route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );

    surface.capture_pointer(UiNodeId::new(3)).unwrap();
    let moved = surface
        .route_pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0))
        .unwrap();
    assert_eq!(moved.target, Some(UiNodeId::new(3)));
    assert_eq!(moved.stacked, vec![UiNodeId::new(2), UiNodeId::new(1)]);
    assert_eq!(moved.entered, vec![UiNodeId::new(2)]);
    assert_eq!(moved.left, vec![UiNodeId::new(3)]);
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));

    let up = surface
        .route_pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
        .unwrap();
    assert_eq!(up.target, Some(UiNodeId::new(3)));
    assert_eq!(surface.focus.captured, None);
}
