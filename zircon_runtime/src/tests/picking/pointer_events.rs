use super::*;

#[test]
fn pointer_event_state_emits_hover_transitions_before_move() {
    let pointer = PointerId::new(1);
    let location = pointer_location(pointer, 10.0, 10.0);
    let mut state = PickingEventState::default();
    let current = PickingHoverMap::new(pointer, vec![hit(HitTarget::renderable(1), 0.1)]);

    let events = state.dispatch_frame(
        current,
        &[location],
        &[PointerInput::new(
            location,
            PointerAction::Move {
                delta: Vec2::new(1.0, 0.0),
            },
        )],
    );

    assert_eq!(
        event_labels(&events),
        vec![
            PickingEventLabel::Enter,
            PickingEventLabel::Over,
            PickingEventLabel::Move,
        ]
    );
    assert_eq!(events[0].propagate, false);
    assert_eq!(events[1].target, HitTarget::renderable(1));
}

#[test]
fn pointer_event_state_click_release_use_previous_hover() {
    let pointer = PointerId::new(1);
    let start = pointer_location(pointer, 10.0, 10.0);
    let release = pointer_location(pointer, 90.0, 90.0);
    let target = HitTarget::renderable(1);
    let mut state = PickingEventState::default();

    let first = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(target, 0.1)]),
        &[start],
        &[PointerInput::new(
            start,
            PointerAction::Press(PointerButton::Primary),
        )],
    );
    assert_eq!(
        event_labels(&first),
        vec![
            PickingEventLabel::Enter,
            PickingEventLabel::Over,
            PickingEventLabel::Press,
        ]
    );

    let second = state.dispatch_frame(
        PickingHoverMap::default(),
        &[release],
        &[PointerInput::new(
            release,
            PointerAction::Release(PointerButton::Primary),
        )],
    );

    assert_eq!(
        event_labels(&second),
        vec![
            PickingEventLabel::Out,
            PickingEventLabel::Leave,
            PickingEventLabel::Click,
            PickingEventLabel::Release,
        ]
    );
    assert!(second.iter().all(|event| event.target == target));
}

#[test]
fn pointer_event_state_drag_drop_and_scroll_sequence() {
    let pointer = PointerId::new(1);
    let dragged = HitTarget::handle_axis(1, PickingAxis::X);
    let drop_target = HitTarget::renderable(2);
    let start = pointer_location(pointer, 10.0, 10.0);
    let drag_location = pointer_location(pointer, 20.0, 10.0);
    let release = pointer_location(pointer, 25.0, 10.0);
    let mut state = PickingEventState::default();

    state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(dragged, 0.1)]),
        &[start],
        &[PointerInput::new(
            start,
            PointerAction::Press(PointerButton::Primary),
        )],
    );

    let drag_events = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(dragged, 0.1), hit(drop_target, 0.2)]),
        &[drag_location],
        &[PointerInput::new(
            drag_location,
            PointerAction::Move {
                delta: Vec2::new(10.0, 0.0),
            },
        )],
    );
    assert_eq!(
        event_labels(&drag_events),
        vec![
            PickingEventLabel::Enter,
            PickingEventLabel::Over,
            PickingEventLabel::DragStart,
            PickingEventLabel::DragEnter,
            PickingEventLabel::Drag,
            PickingEventLabel::DragOver,
            PickingEventLabel::Move,
            PickingEventLabel::Move,
        ]
    );
    assert!(drag_events.iter().any(|event| matches!(
        event.kind,
        PickingEventKind::DragOver { dragged: target, .. } if target == dragged
    )));

    let release_events = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(drop_target, 0.1)]),
        &[release],
        &[
            PointerInput::new(
                release,
                PointerAction::Scroll {
                    unit: PointerScrollUnit::Pixel,
                    delta: Vec2::new(0.0, -4.0),
                },
            ),
            PointerInput::new(release, PointerAction::Release(PointerButton::Primary)),
        ],
    );

    assert_eq!(
        event_labels(&release_events),
        vec![
            PickingEventLabel::Out,
            PickingEventLabel::Leave,
            PickingEventLabel::DragLeave,
            PickingEventLabel::Scroll,
            PickingEventLabel::Click,
            PickingEventLabel::Release,
            PickingEventLabel::Release,
            PickingEventLabel::DragDrop,
            PickingEventLabel::DragEnd,
            PickingEventLabel::DragLeave,
        ]
    );
    assert!(release_events.iter().any(|event| matches!(
        event.kind,
        PickingEventKind::DragDrop { dropped: target, .. } if target == dragged
    )));
}

#[test]
fn pointer_event_state_cancel_filters_current_hover_and_clears_state() {
    let pointer = PointerId::new(1);
    let previous_target = HitTarget::renderable(1);
    let current_target = HitTarget::renderable(2);
    let start = pointer_location(pointer, 10.0, 10.0);
    let cancel = pointer_location(pointer, 20.0, 20.0);
    let mut state = PickingEventState::default();

    state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(previous_target, 0.1)]),
        &[start],
        &[PointerInput::new(
            start,
            PointerAction::Press(PointerButton::Primary),
        )],
    );

    let cancel_events = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(current_target, 0.1)]),
        &[cancel],
        &[PointerInput::new(cancel, PointerAction::Cancel)],
    );

    assert_eq!(
        event_labels(&cancel_events),
        vec![
            PickingEventLabel::Out,
            PickingEventLabel::Leave,
            PickingEventLabel::Cancel,
        ]
    );
    assert!(cancel_events
        .iter()
        .all(|event| event.target == previous_target));

    let release_after_cancel = state.dispatch_frame(
        PickingHoverMap::default(),
        &[cancel],
        &[PointerInput::new(
            cancel,
            PointerAction::Release(PointerButton::Primary),
        )],
    );
    assert!(release_after_cancel.is_empty());
}
