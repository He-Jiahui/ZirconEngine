use super::*;

#[test]
fn navigation_routes_from_focus_and_falls_back_to_roots() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button")).with_state_flags(
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                },
            ),
        )
        .unwrap();
    surface.focus = UiFocusState {
        focused: Some(UiNodeId::new(2)),
        captured: None,
        pressed: None,
        hovered: Vec::new(),
        ..UiFocusState::default()
    };

    let focused = surface
        .route_navigation_event(UiNavigationEventKind::Next)
        .unwrap();
    assert_eq!(focused.target, Some(UiNodeId::new(2)));
    assert_eq!(focused.bubbled, vec![UiNodeId::new(2), UiNodeId::new(1)]);
    assert!(!focused.fallback_to_root);

    surface.focus.focused = None;
    let fallback = surface
        .route_navigation_event(UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(fallback.target, None);
    assert!(fallback.fallback_to_root);
    assert_eq!(fallback.root_targets, vec![UiNodeId::new(1)]);
}

#[test]
fn navigation_dispatcher_bubbles_from_focus_and_can_move_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/left")).with_state_flags(
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/right")).with_state_flags(
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                },
            ),
        )
        .unwrap();
    surface.focus = UiFocusState {
        focused: Some(UiNodeId::new(2)),
        captured: None,
        pressed: None,
        hovered: Vec::new(),
        ..UiFocusState::default()
    };

    let mut dispatcher = UiNavigationDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiNavigationEventKind::Next, |_context| {
        UiNavigationDispatchEffect::Unhandled
    });
    dispatcher.register(UiNodeId::new(1), UiNavigationEventKind::Next, |_context| {
        UiNavigationDispatchEffect::focus(UiNodeId::new(3))
    });

    let result = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
        .unwrap();

    assert_eq!(result.route.target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.route.bubbled,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.handled_by, Some(UiNodeId::new(1)));
    assert_eq!(result.focus_changed_to, Some(UiNodeId::new(3)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
}

#[test]
fn navigation_dispatcher_falls_back_to_root_handlers_without_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
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

    let mut dispatcher = UiNavigationDispatcher::default();
    dispatcher.register(
        UiNodeId::new(1),
        UiNavigationEventKind::Activate,
        |_context| UiNavigationDispatchEffect::handled(),
    );

    let result = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Activate)
        .unwrap();

    assert!(result.route.fallback_to_root);
    assert_eq!(result.route.root_targets, vec![UiNodeId::new(1)]);
    assert_eq!(result.handled_by, Some(UiNodeId::new(1)));
    assert_eq!(surface.focus.focused, None);
}

#[test]
fn navigation_dispatcher_falls_back_to_shared_tab_order_when_unhandled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 80.0))
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
    for (node_id, x) in [(2, 0.0), (3, 80.0), (4, 160.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(node_id),
                    UiNodePath::new(format!("root/item_{node_id}")),
                )
                .with_frame(UiFrame::new(x, 0.0, 60.0, 40.0))
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
    }

    let dispatcher = UiNavigationDispatcher::default();

    let first = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
        .unwrap();
    assert!(first.route.fallback_to_root);
    assert_eq!(first.focus_changed_to, Some(UiNodeId::new(2)));
    assert_eq!(first.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    surface.focus.focused = Some(UiNodeId::new(4));
    let wrapped = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
        .unwrap();
    assert_eq!(wrapped.route.target, Some(UiNodeId::new(4)));
    assert_eq!(wrapped.focus_changed_to, Some(UiNodeId::new(2)));
    assert_eq!(wrapped.handled_by, Some(UiNodeId::new(4)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    let previous = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Previous)
        .unwrap();
    assert_eq!(previous.route.target, Some(UiNodeId::new(2)));
    assert_eq!(previous.focus_changed_to, Some(UiNodeId::new(4)));
    assert_eq!(previous.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
}

#[test]
fn navigation_dispatcher_falls_back_to_nearest_directional_focus_target() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 220.0))
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
    for (node_id, frame) in [
        (2, UiFrame::new(10.0, 10.0, 40.0, 40.0)),
        (3, UiFrame::new(90.0, 20.0, 40.0, 40.0)),
        (4, UiFrame::new(20.0, 100.0, 40.0, 40.0)),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(node_id),
                    UiNodePath::new(format!("root/item_{node_id}")),
                )
                .with_frame(frame)
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
    }
    surface.focus.focused = Some(UiNodeId::new(2));

    let dispatcher = UiNavigationDispatcher::default();

    let right = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Right)
        .unwrap();
    assert_eq!(right.route.target, Some(UiNodeId::new(2)));
    assert_eq!(right.focus_changed_to, Some(UiNodeId::new(3)));
    assert_eq!(right.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));

    surface.focus.focused = Some(UiNodeId::new(2));
    let down = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Down)
        .unwrap();
    assert_eq!(down.route.target, Some(UiNodeId::new(2)));
    assert_eq!(down.focus_changed_to, Some(UiNodeId::new(4)));
    assert_eq!(down.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
}

#[test]
fn navigation_dispatcher_starts_directional_fallback_from_shared_endcaps_without_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 80.0))
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
    for (node_id, x) in [(2, 0.0), (3, 80.0), (4, 160.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(node_id),
                    UiNodePath::new(format!("root/item_{node_id}")),
                )
                .with_frame(UiFrame::new(x, 0.0, 60.0, 40.0))
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
    }

    let dispatcher = UiNavigationDispatcher::default();

    let right = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Right)
        .unwrap();
    assert!(right.route.fallback_to_root);
    assert_eq!(right.focus_changed_to, Some(UiNodeId::new(2)));
    assert_eq!(right.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    surface.clear_focus();
    let left = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Left)
        .unwrap();
    assert!(left.route.fallback_to_root);
    assert_eq!(left.focus_changed_to, Some(UiNodeId::new(4)));
    assert_eq!(left.handled_by, Some(UiNodeId::new(4)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
}

#[test]
fn navigation_dispatcher_keeps_focus_when_activate_or_cancel_is_unhandled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 40.0))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/item"))
                .with_frame(UiFrame::new(0.0, 0.0, 60.0, 40.0))
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
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let dispatcher = UiNavigationDispatcher::default();

    let activate = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(activate.route.target, Some(UiNodeId::new(2)));
    assert_eq!(activate.focus_changed_to, None);
    assert_eq!(activate.handled_by, None);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    let cancel = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Cancel)
        .unwrap();
    assert_eq!(cancel.route.target, Some(UiNodeId::new(2)));
    assert_eq!(cancel.focus_changed_to, None);
    assert_eq!(cancel.handled_by, None);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
}
