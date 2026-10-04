use super::*;
use crate::ui::dispatch::UiInputManager;
use zircon_runtime_interface::ui::window::{
    UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpEvent,
};

fn send(
    surface: &mut UiSurface,
    dispatcher: &UiPointerDispatcher,
    pointer: u64,
    button: UiPointerButton,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            dispatcher,
            &UiNavigationDispatcher::default(),
            owned_pointer_event(pointer, button, kind, point),
        )
        .unwrap()
}

// 先走真实 Down 回调取得两份捕获，保存 Primary 按钮资格，供后续生命周期测试复用。
fn captured_pair() -> (UiSurface, UiPointerDispatcher) {
    let mut surface = route_surface();
    let mut dispatcher = UiPointerDispatcher::default();
    for node in [2, 3] {
        dispatcher.register(UiNodeId::new(node), UiPointerEventKind::Down, |_| {
            UiPointerDispatchEffect::capture()
        });
        dispatcher.register(UiNodeId::new(node), UiPointerEventKind::Move, |_| {
            UiPointerDispatchEffect::handled()
        });
    }
    send(
        &mut surface,
        &dispatcher,
        11,
        UiPointerButton::Primary,
        UiPointerEventKind::Down,
        UiPoint::new(20.0, 20.0),
    );
    send(
        &mut surface,
        &dispatcher,
        12,
        UiPointerButton::Primary,
        UiPointerEventKind::Down,
        UiPoint::new(20.0, 60.0),
    );
    assert_pointer_capture(&surface, UiPointerId::new(11), UiNodeId::new(2));
    assert_pointer_capture(&surface, UiPointerId::new(12), UiNodeId::new(3));
    (surface, dispatcher)
}

#[test]
fn pointer_button_ownership_same_node_press_survives_other_pointer_up_and_cancel() {
    for terminal in [UiPointerEventKind::Up, UiPointerEventKind::Cancel] {
        let mut surface = route_surface();
        let dispatcher = UiPointerDispatcher::default();
        for pointer in [11, 12] {
            send(
                &mut surface,
                &dispatcher,
                pointer,
                UiPointerButton::Primary,
                UiPointerEventKind::Down,
                UiPoint::new(20.0, 20.0),
            );
        }
        send(
            &mut surface,
            &dispatcher,
            12,
            UiPointerButton::Primary,
            terminal,
            UiPoint::new(20.0, 20.0),
        );
        assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
        assert!(
            surface
                .tree
                .node(UiNodeId::new(2))
                .unwrap()
                .state_flags
                .pressed
        );
        assert!(
            surface
                .component_state(UiNodeId::new(2))
                .unwrap()
                .flags
                .pressed
        );
        send(
            &mut surface,
            &dispatcher,
            11,
            UiPointerButton::Primary,
            UiPointerEventKind::Up,
            UiPoint::new(20.0, 20.0),
        );
        assert_eq!(surface.focus.pressed, None);
        assert!(
            !surface
                .tree
                .node(UiNodeId::new(2))
                .unwrap()
                .state_flags
                .pressed
        );
        assert!(surface.input.pointer_presses.is_empty());
    }
}

#[test]
fn pointer_button_ownership_foreign_pointer_press_does_not_borrow_another_capture() {
    let (mut surface, dispatcher) = captured_pair();
    let foreign = send(
        &mut surface,
        &dispatcher,
        13,
        UiPointerButton::Secondary,
        UiPointerEventKind::Down,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(foreign.diagnostics.route_trace.capture_target, None);
    send(
        &mut surface,
        &dispatcher,
        13,
        UiPointerButton::Primary,
        UiPointerEventKind::Up,
        UiPoint::new(20.0, 20.0),
    );
    assert_pointer_capture(&surface, UiPointerId::new(11), UiNodeId::new(2));
    assert_pointer_capture(&surface, UiPointerId::new(12), UiNodeId::new(3));
    assert_eq!(
        surface.input.pointer_press_owner(UiPointerId::new(11)),
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        surface.input.pointer_press_owner(UiPointerId::new(12)),
        Some(UiNodeId::new(3))
    );
}

#[test]
fn pointer_button_ownership_foreign_reply_cannot_transfer_capture_or_emit_host_requests() {
    let (mut surface, dispatcher) = captured_pair();
    let before_input = surface.input.clone();
    let before_focus = surface.focus.clone();
    let rejected = surface.apply_dispatch_reply(
        owned_pointer_event(
            11,
            UiPointerButton::Secondary,
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 60.0),
        ),
        UiDispatchReply::handled().with_effects([
            UiDispatchEffect::CapturePointer {
                target: UiNodeId::new(3),
                pointer_id: UiPointerId::new(11),
                reason: UiPointerCaptureReason::Press,
            },
            UiDispatchEffect::UseHighPrecisionPointer {
                target: UiNodeId::new(3),
                enabled: true,
            },
        ]),
    );
    assert_eq!(rejected.rejected_effects.len(), 2);
    assert!(rejected.applied_effects.is_empty());
    assert!(rejected.host_requests.is_empty());
    assert_eq!(surface.input, before_input);
    assert_eq!(surface.focus, before_focus);
    send(
        &mut surface,
        &dispatcher,
        11,
        UiPointerButton::Secondary,
        UiPointerEventKind::Up,
        UiPoint::new(200.0, 200.0),
    );
    assert_pointer_capture(&surface, UiPointerId::new(11), UiNodeId::new(2));
    send(
        &mut surface,
        &dispatcher,
        11,
        UiPointerButton::Primary,
        UiPointerEventKind::Up,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(
        surface.input.pointer_capture_owner(UiPointerId::new(11)),
        None
    );
    assert_pointer_capture(&surface, UiPointerId::new(12), UiNodeId::new(3));
}

#[test]
// 最后的 Middle Up 检查事务回滚同时恢复 owner 与 Primary 按钮资格，避免错误释放捕获。
fn pointer_button_ownership_atomic_reply_restores_qualified_capture_after_later_failure() {
    let (mut surface, dispatcher) = captured_pair();
    let before_input = surface.input.clone();
    let before_focus = surface.focus.clone();
    let rejected = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effects([
            UiDispatchEffect::ReleasePointerCapture {
                target: UiNodeId::new(2),
                pointer_id: UiPointerId::new(11),
                reason: UiPointerCaptureReason::Cancel,
            },
            UiDispatchEffect::CapturePointer {
                target: UiNodeId::new(999),
                pointer_id: UiPointerId::new(11),
                reason: UiPointerCaptureReason::Press,
            },
        ]),
    );
    assert_eq!(rejected.rejected_effects.len(), 2);
    assert!(rejected.applied_effects.is_empty());
    assert!(rejected.host_requests.is_empty());
    assert_eq!(surface.input, before_input);
    assert_eq!(surface.focus, before_focus);
    send(
        &mut surface,
        &dispatcher,
        11,
        UiPointerButton::Middle,
        UiPointerEventKind::Up,
        UiPoint::new(200.0, 200.0),
    );
    assert_pointer_capture(&surface, UiPointerId::new(11), UiNodeId::new(2));
}

#[test]
// 用键盘事件应用捕获转移，使新捕获不携带按钮资格；Middle Up 可释放捕获，原 Primary 按压仍由原节点持有。
fn pointer_button_ownership_programmatic_transfer_is_unqualified_and_release_keeps_press() {
    let (mut surface, dispatcher) = captured_pair();
    let moved = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(3),
            pointer_id: UiPointerId::new(11),
            reason: UiPointerCaptureReason::Press,
        }),
    );
    assert!(moved.rejected_effects.is_empty());
    send(
        &mut surface,
        &dispatcher,
        11,
        UiPointerButton::Middle,
        UiPointerEventKind::Up,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(
        surface.input.pointer_capture_owner(UiPointerId::new(11)),
        None
    );
    assert_eq!(
        surface.input.pointer_press_owner(UiPointerId::new(11)),
        Some(UiNodeId::new(2))
    );
    let released = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(3),
            pointer_id: UiPointerId::new(12),
            reason: UiPointerCaptureReason::Cancel,
        }),
    );
    assert!(released.rejected_effects.is_empty());
    assert_eq!(
        surface.input.pointer_press_owner(UiPointerId::new(12)),
        Some(UiNodeId::new(3))
    );
    send(
        &mut surface,
        &dispatcher,
        11,
        UiPointerButton::Primary,
        UiPointerEventKind::Up,
        UiPoint::new(20.0, 20.0),
    );
    send(
        &mut surface,
        &dispatcher,
        12,
        UiPointerButton::Primary,
        UiPointerEventKind::Up,
        UiPoint::new(20.0, 60.0),
    );
    assert!(surface.input.pointer_presses.is_empty());
}

#[test]
fn pointer_button_ownership_detach_and_disable_clear_only_the_invalid_owner() {
    for detach in [true, false] {
        let (mut surface, dispatcher) = captured_pair();
        if detach {
            surface.detach_subtree_to_pool(UiNodeId::new(2)).unwrap();
        } else {
            surface
                .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
                    UiNodeId::new(2),
                    "enabled",
                    UiValue::Bool(false),
                ))
                .unwrap();
            assert!(
                !surface
                    .tree
                    .node(UiNodeId::new(2))
                    .unwrap()
                    .state_flags
                    .pressed
            );
            assert!(
                !surface
                    .component_state(UiNodeId::new(2))
                    .unwrap()
                    .flags
                    .pressed
            );
            surface
                .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
                    UiNodeId::new(2),
                    "enabled",
                    UiValue::Bool(true),
                ))
                .unwrap();
            assert!(
                !surface
                    .tree
                    .node(UiNodeId::new(2))
                    .unwrap()
                    .state_flags
                    .pressed
            );
            assert!(
                !surface
                    .component_state(UiNodeId::new(2))
                    .unwrap()
                    .flags
                    .pressed
            );
        }
        assert_eq!(
            surface.input.pointer_capture_owner(UiPointerId::new(11)),
            None
        );
        assert_eq!(
            surface.input.pointer_press_owner(UiPointerId::new(11)),
            None
        );
        assert_pointer_capture(&surface, UiPointerId::new(12), UiNodeId::new(3));
        assert!(
            surface
                .tree
                .node(UiNodeId::new(3))
                .unwrap()
                .state_flags
                .pressed
        );
        assert!(
            surface
                .component_state(UiNodeId::new(3))
                .unwrap()
                .flags
                .pressed
        );
        assert_eq!(
            surface.input.pointer_press_owner(UiPointerId::new(12)),
            Some(UiNodeId::new(3))
        );
        let moved = send(
            &mut surface,
            &dispatcher,
            12,
            UiPointerButton::Primary,
            UiPointerEventKind::Move,
            UiPoint::new(200.0, 200.0),
        );
        assert_eq!(moved.reply.handler, Some(UiNodeId::new(3)));
    }
}

#[test]
fn pointer_button_ownership_window_blur_deactivate_and_destroy_clear_all_pointers() {
    for kind in [
        UiWindowEventKind::Focused { focused: false },
        UiWindowEventKind::ApplicationActivation { is_active: false },
        UiWindowEventKind::Destroyed,
    ] {
        let (mut surface, _) = captured_pair();
        surface
            .dispatch_window_input_pump_event(
                &mut UiInputManager::default(),
                UiWindowInputPumpEvent::Window(UiWindowEvent::new(
                    UiWindowEventMetadata::default(),
                    kind,
                )),
            )
            .unwrap();
        assert!(surface.input.pointer_captures.is_empty());
        assert!(surface.input.pointer_presses.is_empty());
        assert_eq!(surface.focus.captured, None);
        assert_eq!(surface.focus.pressed, None);
        for owner in [UiNodeId::new(2), UiNodeId::new(3)] {
            assert!(!surface.component_state(owner).unwrap().flags.pressed);
            assert!(!surface.tree.node(owner).unwrap().state_flags.pressed);
        }
    }
}

#[test]
fn pointer_button_ownership_hot_reload_drops_old_press_and_capture_qualification() {
    let (old, _) = captured_pair();
    let mut replacement = route_surface();
    replacement.adopt_hot_reload_state_from(&old);
    assert!(replacement.input.pointer_captures.is_empty());
    assert!(replacement.input.pointer_presses.is_empty());
    assert_eq!(replacement.focus.pressed, None);
    assert_eq!(replacement.focus.captured, None);
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::capture()
    });
    send(
        &mut replacement,
        &dispatcher,
        11,
        UiPointerButton::Secondary,
        UiPointerEventKind::Down,
        UiPoint::new(20.0, 20.0),
    );
    send(
        &mut replacement,
        &dispatcher,
        11,
        UiPointerButton::Primary,
        UiPointerEventKind::Up,
        UiPoint::new(200.0, 200.0),
    );
    assert_pointer_capture(&replacement, UiPointerId::new(11), UiNodeId::new(2));
    send(
        &mut replacement,
        &dispatcher,
        11,
        UiPointerButton::Secondary,
        UiPointerEventKind::Up,
        UiPoint::new(200.0, 200.0),
    );
    assert_no_pointer_capture(&replacement);
}
