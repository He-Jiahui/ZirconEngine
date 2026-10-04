use super::*;

#[test]
fn pointer_dispatcher_applies_block_passthrough_and_capture_semantics() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/bottom"))
                .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
                .with_z_index(0)
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
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/top"))
                .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
                .with_z_index(10)
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
    surface.rebuild();

    let mut block_dispatcher = UiPointerDispatcher::default();
    block_dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::blocked()
    });
    block_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });

    let blocked = surface
        .dispatch_pointer_event(
            &block_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(blocked.blocked_by, Some(UiNodeId::new(3)));
    assert_eq!(blocked.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        blocked
            .invocations
            .iter()
            .map(|invocation| (invocation.node_id, invocation.effect))
            .collect::<Vec<_>>(),
        vec![
            (UiNodeId::new(3), UiPointerDispatchEffect::Blocked),
            (UiNodeId::new(2), UiPointerDispatchEffect::Handled),
        ]
    );

    let mut passthrough_dispatcher = UiPointerDispatcher::default();
    passthrough_dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::passthrough()
    });
    passthrough_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });
    let passthrough = surface
        .dispatch_pointer_event(
            &passthrough_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(passthrough.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(passthrough.passthrough, vec![UiNodeId::new(3)]);

    let mut capture_dispatcher = UiPointerDispatcher::default();
    capture_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::capture()
    });
    let captured = surface
        .dispatch_pointer_event(
            &capture_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(captured.captured_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
}

#[test]
fn captured_pointer_dispatch_keeps_move_and_up_targeting_the_captured_node_outside_hit_bounds() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/viewport"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0))
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

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::capture()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::handled()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Up, |_context| {
        UiPointerDispatchEffect::handled()
    });

    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(down.captured_by, Some(UiNodeId::new(2)));

    let moved = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(160.0, 160.0)),
        )
        .unwrap();
    assert_eq!(moved.route.target, Some(UiNodeId::new(2)));
    assert_eq!(moved.handled_by, Some(UiNodeId::new(2)));

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(160.0, 160.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(up.route.target, Some(UiNodeId::new(2)));
    assert_eq!(up.handled_by, Some(UiNodeId::new(2)));
}

#[test]
fn scroll_pointer_event_scrolls_the_nearest_scrollable_box_when_unhandled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
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
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
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
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let result = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        50.0
    );
    assert!(surface.tree.node(UiNodeId::new(2)).unwrap().dirty.layout);

    let scratch_capacity = surface.default_scroll_candidate_scratch_capacity_for_test();
    assert!(scratch_capacity > 0);
    let repeated = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(10.0),
        )
        .unwrap();
    assert_eq!(repeated.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        surface.default_scroll_candidate_scratch_capacity_for_test(),
        scratch_capacity
    );
}

#[test]
fn default_pointer_scroll_validates_every_candidate_before_mutating_and_recovers_scratch() {
    let mut surface = UiSurface::new(UiTreeId::new("default-scroll-validation"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("scroll"))
            .with_container(UiContainerKind::ScrollableBox(
                UiScrollableBoxConfig::default(),
            ))
            .with_scroll_state(UiScrollState {
                offset: 0.0,
                viewport_extent: 100.0,
                content_extent: 200.0,
            }),
    );

    assert!(matches!(
        surface.apply_default_pointer_scroll(&default_scroll_route(vec![
            UiNodeId::new(1),
            UiNodeId::new(99),
        ])),
        Err(UiTreeError::MissingNode(UiNodeId(99)))
    ));
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );

    let scratch_capacity = surface.default_scroll_candidate_scratch_capacity_for_test();
    assert!(scratch_capacity > 0);
    assert_eq!(
        surface
            .apply_default_pointer_scroll(&default_scroll_route(vec![UiNodeId::new(1)]))
            .unwrap(),
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        surface.default_scroll_candidate_scratch_capacity_for_test(),
        scratch_capacity
    );
}

#[test]
fn default_pointer_scroll_reuses_a_surface_local_candidate_scratch() {
    let source = include_str!("../../../surface/surface/event_routing.rs");
    let scroll_start = source
        .find("    pub fn apply_default_pointer_scroll(")
        .expect("default pointer scroll");
    let scroll_end = source[scroll_start..]
        .find("\n    pub fn route_pointer_event_with_query(")
        .map(|offset| scroll_start + offset)
        .expect("next event routing method");
    let scroll = &source[scroll_start..scroll_end];

    assert!(scroll.contains("std::mem::take(&mut self.scrollable_candidate_scratch)"));
    assert!(
        scroll.contains("collect_scrollable_candidates(candidates, &mut scrollable_candidates)")
    );
    assert!(scroll.contains("self.scrollable_candidate_scratch = scrollable_candidates"));
    assert!(
        scroll.find("collect_scrollable_candidates") < scroll.find("self.tree.scroll_by"),
        "the complete candidate validation must run before the first scroll mutation"
    );
    assert!(!scroll.contains("self.tree.scrollable_candidates(candidates)?"));
}

fn default_scroll_route(stacked: Vec<UiNodeId>) -> UiPointerRoute {
    UiPointerRoute {
        kind: UiPointerEventKind::Scroll,
        button: None,
        modifiers: Default::default(),
        activation_phase: Default::default(),
        point: UiPoint::new(0.0, 0.0),
        scroll_delta: 1.0,
        target: None,
        hit_path: Default::default(),
        routing_path: Default::default(),
        stacked,
        entered: Vec::new(),
        left: Vec::new(),
        captured: None,
        pressed: None,
        click_target: None,
        release_inside_pressed: false,
        focused: None,
        fallback_to_root: false,
        root_targets: Vec::new(),
    }
}
