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
