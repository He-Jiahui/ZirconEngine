use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityActionRequest},
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase,
        UiDispatchReply, UiDispatchReplyStep, UiDragDropInputEvent, UiDragDropInputEventKind,
        UiDragSessionId, UiFocusEffectReason, UiImeInputEvent, UiImeInputEventKind,
        UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiInputRoutePolicy,
        UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
        UiNavigationInputEvent, UiPointerCaptureReason, UiPointerEvent, UiPointerId,
        UiPointerInputEvent, UiPointerSource, UiPopupEffectKind, UiPopupInputEvent,
        UiPopupInputEventKind, UiPreciseScrollDelta, UiTextInputEvent, UiTooltipEffectKind,
        UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    focus::UiFocusedInputKind,
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
    },
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

mod keyboard_activation_routes;
mod keyboard_navigation_routes;
mod keyboard_popup_routes;
mod pointer_hover_routes;
mod pointer_popup_routes;
mod touch_pointer_routes;

#[test]
fn direct_dispatch_reply_populates_focus_route_trace_after_effects() {
    let mut surface = route_surface();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_start, 0);
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
}

#[test]
fn dispatch_reply_steps_report_stopped_preview_and_focus_trace() {
    let mut surface = route_surface();

    let result = surface.apply_dispatch_reply_steps(
        keyboard_event(),
        [
            UiDispatchReplyStep::new(
                UiDispatchPhase::Preprocess,
                None,
                UiDispatchReply::unhandled(),
            ),
            UiDispatchReplyStep::new(
                UiDispatchPhase::PreviewTunnel,
                Some(UiNodeId::new(1)),
                UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
                    target: UiNodeId::new(2),
                    reason: UiFocusEffectReason::Input,
                }),
            ),
            UiDispatchReplyStep::new(
                UiDispatchPhase::Bubble,
                Some(UiNodeId::new(3)),
                UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
                    target: UiNodeId::new(3),
                    reason: UiFocusEffectReason::Input,
                }),
            ),
        ],
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.applied_effects.len(), 1);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase,
        Some("preview_tunnel".to_string())
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(1)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "dispatch_steps=2"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "propagation_stopped"));
    assert_eq!(result.diagnostics.route_steps.len(), 2);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Preprocess
    );
    assert_eq!(result.diagnostics.route_steps[0].target, None);
    assert_eq!(result.diagnostics.route_steps[0].handler, None);
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_start, 0);
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 0);
    assert!(!result.diagnostics.route_steps[0].stopped);
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].handler,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[1].effect_start, 0);
    assert_eq!(result.diagnostics.route_steps[1].effect_count, 1);
    assert!(result.diagnostics.route_steps[1].stopped);
}

#[test]
fn unified_pointer_dispatch_reports_phase_route_steps() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(result.reply.effects.is_empty());
    assert_eq!(result.diagnostics.route_steps.len(), 4);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert!(!result.diagnostics.route_steps[2].stopped);
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Passthrough
    );
}

#[test]
fn unified_pointer_press_release_report_bubble_route_steps_and_component_events() {
    let mut surface = press_release_route_surface();

    let down = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_two_node_bubble_handled_at_target(&down);
    assert_eq!(down.component_events.len(), 1);
    assert_eq!(down.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        down.component_events[0].event,
        UiComponentEvent::Press { pressed: true }
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

    let up = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_two_node_bubble_handled_at_target(&up);
    assert_eq!(up.component_events.len(), 2);
    assert_eq!(up.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        up.component_events[0].event,
        UiComponentEvent::Press { pressed: false }
    );
    assert_eq!(up.component_events[1].target, UiNodeId::new(2));
    assert_eq!(
        up.component_events[1].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
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
}

#[test]
fn unified_pointer_double_click_reports_bubble_route_steps_and_default_binding() {
    let mut surface = double_click_route_surface();

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event_with_click_count(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0), 2),
        )
        .unwrap();

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.reply.effects.is_empty());
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "double_activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => assert_eq!(pointer.event.click_count, 2),
        _ => panic!("expected pointer input event"),
    }
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert!(result.diagnostics.route_steps[2].stopped);
}

#[test]
fn unified_pointer_scroll_reports_bubble_route_steps_and_precise_delta() {
    let mut surface = scroll_route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            scroll_event(UiPoint::new(20.0, 20.0), 50.0),
        )
        .unwrap();

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(20)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "scroll_delta=50"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(20)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(20), UiNodeId::new(2), UiNodeId::new(1)]
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.event.scroll_delta, 50.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::pixels(0.0, 50.0))
            );
        }
        _ => panic!("expected pointer input event"),
    }
    assert_eq!(result.diagnostics.route_steps.len(), 5);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert!(!result.diagnostics.route_steps[3].stopped);
    assert_eq!(
        result.diagnostics.route_steps[4].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        result.diagnostics.route_steps[4].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[4].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[4].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[4].effect_count, 0);
    assert!(result.diagnostics.route_steps[4].stopped);
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
}

#[test]
fn unified_focus_and_capture_dispatch_report_phase_route_steps() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let keyboard = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_event(),
        )
        .unwrap();

    assert_eq!(
        keyboard.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(keyboard.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(keyboard.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        keyboard.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        keyboard.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        keyboard.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(keyboard.diagnostics.route_steps.len(), 4);
    assert_eq!(
        keyboard.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(!keyboard.diagnostics.route_steps[2].stopped);
    assert_eq!(
        keyboard.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);

    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));
    let captured = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 60.0)),
        )
        .unwrap();

    assert_eq!(
        captured.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        captured.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(captured.diagnostics.route_steps.len(), 1);
    assert_eq!(
        captured.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        captured.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn unified_navigation_dispatch_reports_route_steps_and_focused_input_log() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            navigation_event(UiNavigationEventKind::Next),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(3)
                && *reason == UiFocusEffectReason::Navigation
    ));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("navigation")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_text_and_ime_dispatch_report_focus_route_steps_and_focused_input_log() {
    let mut surface = editable_route_surface("Hi", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let text = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            text_event("!"),
        )
        .unwrap();

    assert_eq!(text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text.diagnostics.route_policy, UiInputRoutePolicy::FocusPath);
    assert_eq!(text.diagnostics.handled_phase.as_deref(), Some("text.edit"));
    assert_eq!(text.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(text.diagnostics.route_trace.target, Some(UiNodeId::new(2)));
    assert_eq!(
        text.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        text.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        text.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(text.diagnostics.route_steps.len(), 3);
    assert_eq!(
        text.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        text.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        text.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        text.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        text.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        text.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(text.diagnostics.route_steps[2].effect_count, 0);
    assert!(text.diagnostics.route_steps[2].stopped);
    assert_eq!(editable_attr_string(&surface, "value"), "Hi!");
    assert_eq!(text.component_events.len(), 1);
    assert_eq!(
        text.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Hi!".to_string()),
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Text
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let ime = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            ime_event(UiImeInputEventKind::Commit, "?"),
        )
        .unwrap();

    assert_eq!(ime.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(ime.diagnostics.route_policy, UiInputRoutePolicy::FocusPath);
    assert_eq!(ime.diagnostics.handled_phase.as_deref(), Some("ime.edit"));
    assert_eq!(ime.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(ime.diagnostics.route_trace.target, Some(UiNodeId::new(2)));
    assert_eq!(
        ime.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        ime.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(ime.diagnostics.route_steps.len(), 3);
    assert_eq!(
        ime.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        ime.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(ime.diagnostics.route_steps[2].effect_count, 0);
    assert!(ime.diagnostics.route_steps[2].stopped);
    assert_eq!(editable_attr_string(&surface, "value"), "Hi!?");
    assert!(ime.component_events.iter().any(|event| {
        event.event
            == UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String("Hi!?".to_string()),
            }
    }));
    assert_eq!(surface.focus.focused_inputs.len(), 2);
    assert_eq!(
        surface.focus.focused_inputs[1].kind,
        UiFocusedInputKind::Ime
    );
    assert_eq!(
        surface.focus.focused_inputs[1].route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[1].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[1].accepted);
}

#[test]
fn captured_pointer_up_preserves_capture_route_trace_after_release() {
    let mut surface = route_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.focus.pressed = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));

    let released = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 60.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(
        released.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        released.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(released.diagnostics.route_steps.len(), 1);
    assert_eq!(
        released.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        released.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert!(released.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason
        } if *target == UiNodeId::new(2)
            && *pointer_id == UiPointerId::new(7)
            && *reason == UiPointerCaptureReason::Cancel
    )));
}

#[test]
fn drag_drop_over_trace_uses_drop_target_path_and_preserves_capture_source() {
    let mut surface = route_surface();
    let session_id = UiDragSessionId::new(42);

    let begin = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Begin,
                Some(session_id),
                UiPoint::new(20.0, 20.0),
            ),
        )
        .unwrap();

    assert!(begin.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(begin.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(
        begin.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );

    let over = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Over,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert!(over.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(over.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(over.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(over.diagnostics.route_trace.target, Some(UiNodeId::new(3)));
    assert_eq!(over.diagnostics.route_trace.direct_target, None);
    assert_eq!(
        over.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        over.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(
        over.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(over.diagnostics.route_steps.len(), 3);
    assert_eq!(
        over.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        over.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        over.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        over.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        over.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        over.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        over.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(3))
    );

    let dropped = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Drop,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert!(dropped.rejected_effects.is_empty());
    assert_eq!(dropped.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(
        dropped.diagnostics.route_trace.target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        dropped.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );

    let ended = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::End,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert!(ended.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(
        ended.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(ended.diagnostics.route_trace.target, Some(UiNodeId::new(3)));
    assert_eq!(
        ended.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        ended.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(ended.diagnostics.route_steps.len(), 1);
    assert_eq!(
        ended.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        ended.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn popup_dispatch_reply_trace_includes_capture_and_updated_popup_stack() {
    let mut surface = route_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));

    let result = surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, "menu.file"),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::Popup {
                kind: UiPopupEffectKind::Open,
                popup_id: "menu.file".to_string(),
                owner: Some(UiNodeId::new(2)),
                anchor: Some(UiPoint::new(8.0, 12.0)),
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["menu.file"]
    );
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.popup_stack,
        vec!["menu.file".to_string()]
    );
}

#[test]
fn popup_close_reply_uses_open_popup_owner_for_route_trace() {
    let mut surface = route_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));

    surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, "menu.file"),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "menu.file".to_string(),
            owner: Some(UiNodeId::new(2)),
            anchor: Some(UiPoint::new(8.0, 12.0)),
        }),
    );

    let result = surface.apply_dispatch_reply(
        popup_event_without_owner(UiPopupInputEventKind::CloseRequested, "menu.file"),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::Popup {
                kind: UiPopupEffectKind::Close,
                popup_id: "menu.file".to_string(),
                owner: None,
                anchor: None,
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn tooltip_cancel_reply_uses_armed_tooltip_owner_for_route_trace() {
    let mut surface = route_surface();

    surface.apply_dispatch_reply(
        tooltip_event(
            UiTooltipTimerInputEventKind::Armed,
            "status.hint",
            Some(UiNodeId::new(2)),
        ),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Arm,
            tooltip_id: "status.hint".to_string(),
            owner: Some(UiNodeId::new(2)),
        }),
    );

    let result = surface.apply_dispatch_reply(
        tooltip_event(UiTooltipTimerInputEventKind::Canceled, "status.hint", None),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::Tooltip {
                kind: UiTooltipEffectKind::Cancel,
                tooltip_id: "status.hint".to_string(),
                owner: None,
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(surface.input.tooltip, None);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn tooltip_timer_elapsed_dispatch_reports_owner_default_action_route() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tooltip_event(
                UiTooltipTimerInputEventKind::Elapsed,
                "status.hint",
                Some(UiNodeId::new(2)),
            ),
        )
        .unwrap();

    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| (tooltip.owner, tooltip.visible)),
        Some((Some(UiNodeId::new(2)), true))
    );
    assert_eq!(result.host_requests.len(), 1);
    assert!(matches!(
        result.host_requests[0].request,
        zircon_runtime_interface::ui::dispatch::UiDispatchHostRequestKind::Tooltip {
            kind: UiTooltipEffectKind::Show,
            ref tooltip_id,
        } if tooltip_id == "status.hint"
    ));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("tooltip.effect")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::DefaultAction
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 1);
}

#[test]
fn accessibility_activate_dispatch_reports_owner_default_action_route_steps() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            accessibility_event(UiNodeId::new(2), UiAccessibilityAction::Activate),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::DefaultAction
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 0);
    assert!(result.diagnostics.route_steps[0].stopped);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
}

fn assert_two_node_bubble_handled_at_target(result: &UiInputDispatchResult) {
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert!(result.diagnostics.route_steps[2].stopped);
}

fn route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn press_release_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings: vec![
            binding("MaterialButton/Press", UiEventKind::Press),
            binding("MaterialButton/Release", UiEventKind::Release),
            binding("MaterialButton/Click", UiEventKind::Click),
        ],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn double_click_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings: vec![binding(
            "MaterialButton/DoubleClick",
            UiEventKind::DoubleClick,
        )],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn scroll_route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.scroll"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            },
        ),
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
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
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
                .with_state_flags(input_state()),
            )
            .unwrap();
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();
    surface
}

fn editable_route_surface(value: &str, caret_offset: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.text"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/editable"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    control_id: Some("EditableText".to_string()),
                    bindings: vec![
                        binding("EditableText/Change", UiEventKind::Change),
                        binding("EditableText/Submit", UiEventKind::Submit),
                    ],
                    attributes: toml::from_str(&format!(
                        r#"
value = "{}"
caret_offset = {}
editable_text = true
"#,
                        value, caret_offset
                    ))
                    .unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn input_state() -> UiStateFlags {
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

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: input_metadata(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "KeyA".to_string(),
        text: None,
    })
}

fn text_event(text: &str) -> UiInputEvent {
    UiInputEvent::Text(UiTextInputEvent {
        metadata: input_metadata(),
        text: text.to_string(),
    })
}

fn ime_event(kind: UiImeInputEventKind, text: &str) -> UiInputEvent {
    UiInputEvent::Ime(UiImeInputEvent {
        metadata: input_metadata(),
        kind,
        text: text.to_string(),
        cursor_range: None,
    })
}

fn navigation_event(kind: UiNavigationEventKind) -> UiInputEvent {
    UiInputEvent::Navigation(UiNavigationInputEvent {
        metadata: input_metadata(),
        kind,
    })
}

fn pointer_event(kind: UiPointerEventKind, point: UiPoint) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn pointer_event_with_click_count(
    kind: UiPointerEventKind,
    point: UiPoint,
    click_count: u8,
) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(kind, point)
            .with_button(UiPointerButton::Primary)
            .with_click_count(click_count),
        precise_scroll: None,
    })
}

fn scroll_event(point: UiPoint, scroll_delta: f32) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(UiPointerEventKind::Scroll, point)
            .with_scroll_delta(scroll_delta),
        precise_scroll: Some(UiPreciseScrollDelta::pixels(0.0, scroll_delta)),
    })
}

fn touch_pointer_event_with_id(
    pointer_id: UiPointerId,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.pointer_id = Some(pointer_id);
    metadata.pointer_source = UiPointerSource::Touch;
    touch_pointer_event_from_metadata(metadata, kind, point)
}

fn touch_pointer_event_from_metadata(
    metadata: UiInputEventMetadata,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn drag_drop_event(
    kind: UiDragDropInputEventKind,
    session_id: Option<UiDragSessionId>,
    point: UiPoint,
) -> UiInputEvent {
    UiInputEvent::DragDrop(UiDragDropInputEvent {
        metadata: input_metadata(),
        kind,
        session_id,
        point,
        payload: None,
    })
}

fn popup_event(kind: UiPopupInputEventKind, popup_id: &str) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner: Some(UiNodeId::new(2)),
        anchor: Some(UiPoint::new(8.0, 12.0)),
    })
}

fn popup_event_without_owner(kind: UiPopupInputEventKind, popup_id: &str) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner: None,
        anchor: None,
    })
}

fn tooltip_event(
    kind: UiTooltipTimerInputEventKind,
    tooltip_id: &str,
    owner: Option<UiNodeId>,
) -> UiInputEvent {
    UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
        metadata: input_metadata(),
        kind,
        tooltip_id: tooltip_id.to_string(),
        owner,
    })
}

fn accessibility_event(target: UiNodeId, action: UiAccessibilityAction) -> UiInputEvent {
    UiInputEvent::Accessibility(UiAccessibilityInputEvent {
        metadata: input_metadata(),
        request: UiAccessibilityActionRequest {
            target,
            action,
            ..UiAccessibilityActionRequest::default()
        },
    })
}

fn editable_attr_string(surface: &UiSurface, key: &str) -> String {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
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
