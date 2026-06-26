use super::*;

#[test]
fn table_column_resize_drag_updates_widths_and_emits_value_changed() {
    let mut surface = table_pointer_route_surface(false, false);

    let begin = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(168.0, 22.0),
    );
    assert!(begin.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::BeginDrag {
                    property: "column_width".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::Begin)
    }));
    assert!(begin.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::CapturePointer { target, .. } if *target == UiNodeId::new(2)
    )));

    let update = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(196.0, 22.0),
    );
    assert!(update.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "column_width".to_string(),
                    value: column_width_payload("triangles", 124.0),
                }
    }));
    assert!(update.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::DragDelta {
                    property: "column_width".to_string(),
                    delta: 28.0,
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::Update)
    }));
    assert_table_column_width(&surface, "triangles", 124.0);

    let end = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(196.0, 22.0),
    );
    assert!(end.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::EndDrag {
                    property: "column_width".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::End)
    }));
    assert!(end.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture { target, .. } if *target == UiNodeId::new(2)
    )));
    assert_table_column_width(&surface, "triangles", 124.0);
}

#[test]
fn data_grid_column_resize_drag_updates_mui_grid_widths() {
    let mut surface = table_pointer_route_surface(true, false);

    dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(168.0, 22.0),
    );
    let update = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(200.0, 22.0),
    );

    assert!(update.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "column_width".to_string(),
                    value: column_width_payload("triangles", 128.0),
                }
    }));
    assert_table_column_width(&surface, "triangles", 128.0);
}

#[test]
fn data_grid_disable_column_resize_blocks_default_resize_drag() {
    let mut surface = table_pointer_route_surface(true, true);

    let begin = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(168.0, 22.0),
    );
    dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(210.0, 22.0),
    );
    let end = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(210.0, 22.0),
    );

    assert!(!begin.component_events.iter().any(|event| {
        matches!(event.event, UiComponentEvent::BeginDrag { .. })
            || matches!(event.event, UiComponentEvent::ValueChanged { .. })
    }));
    assert!(!end
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::EndDrag { .. })));
    assert_table_column_width(&surface, "triangles", 96.0);
}
