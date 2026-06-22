use crate::ui::{dispatch::UiInputManager, surface::UiSurface};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchPhase,
        UiDispatchReply, UiInputEvent, UiInputEventMetadata, UiInputRoutePolicy, UiInputSequence,
        UiInputTimestamp, UiMouseMotionInputEvent, UiPointerEvent, UiPointerSource,
        UiPopupEffectKind, UiPopupInputEvent, UiPopupInputEventKind, UiTooltipEffectKind,
        UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind, UiTransientDismissalReason,
        UiTransientDismissalTarget, UiWindowId,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint, UiSize},
    surface::UiPointerEventKind,
    template::UiBindingRef,
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
    window::{
        UiWindowAction, UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata,
        UiWindowInputContext, UiWindowInputPumpBatch, UiWindowInputPumpEvent, UiWindowMetrics,
        UiWindowPixelPosition, UiWindowPixelSize, UiWindowPlatformInputEvent, UiWindowRedrawReason,
    },
};

#[test]
fn window_input_pump_app_deactivation_closes_popup_stack_and_tooltip() {
    let mut surface = route_surface();
    open_popup(&mut surface, "menu.file", UiNodeId::new(2));
    open_popup(&mut surface, "menu.edit", UiNodeId::new(3));
    show_tooltip(&mut surface, "status.hint", UiNodeId::new(2));

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::application_activation_changed(
            window_metadata(7, false),
            false,
        )),
    )
    .unwrap();

    assert_eq!(surface.window_state.application_active, Some(false));
    assert_eq!(
        surface.surface_frame().window_state.application_active,
        Some(false)
    );
    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.input.tooltip.is_none());
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        result.applied_effects[0].effect,
        UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::ApplicationDeactivated,
        }
    ));
    assert_eq!(result.host_requests.len(), 1);
    assert!(matches!(
        result.host_requests[0].request,
        UiDispatchHostRequestKind::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::ApplicationDeactivated,
        }
    ));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some(UiDispatchPhase::DefaultAction.as_str())
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_application_inactive"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump"));
    assert!(matches!(
        result.event,
        UiInputEvent::Popup(UiPopupInputEvent {
            kind: UiPopupInputEventKind::Dismissed,
            ref popup_id,
            ..
        }) if popup_id == "window.transient"
    ));
}

#[test]
fn window_input_pump_focus_loss_closes_popup_stack_and_tooltip() {
    let mut surface = route_surface();
    open_popup(&mut surface, "menu.file", UiNodeId::new(2));
    open_popup(&mut surface, "menu.edit", UiNodeId::new(3));
    show_tooltip(&mut surface, "status.hint", UiNodeId::new(2));

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(
            window_metadata(8, false),
            false,
        )),
    )
    .unwrap();

    assert_eq!(surface.window_state.focused, Some(false));
    assert_eq!(surface.surface_frame().window_state.focused, Some(false));
    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.input.tooltip.is_none());
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        result.applied_effects[0].effect,
        UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::FocusLost,
        }
    ));
    assert_eq!(result.host_requests.len(), 1);
    assert!(matches!(
        result.host_requests[0].request,
        UiDispatchHostRequestKind::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::FocusLost,
        }
    ));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some(UiDispatchPhase::DefaultAction.as_str())
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_focus_lost"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_transient_dismissal"));
    assert!(matches!(
        result.event,
        UiInputEvent::Popup(UiPopupInputEvent {
            kind: UiPopupInputEventKind::Dismissed,
            ref popup_id,
            ..
        }) if popup_id == "window.transient"
    ));
}

#[test]
fn window_input_pump_retains_focus_activation_and_occlusion_facts() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();

    let focused = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(
            window_metadata(23, false),
            true,
        )),
    )
    .unwrap();
    let active = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::application_activation_changed(
            window_metadata(24, false),
            true,
        )),
    )
    .unwrap();
    let occluded = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(25, false),
            UiWindowEventKind::Occluded { occluded: true },
        )),
    )
    .unwrap();

    assert_eq!(surface.window_state.focused, Some(true));
    assert_eq!(surface.window_state.application_active, Some(true));
    assert_eq!(surface.window_state.occluded, Some(true));
    assert_eq!(surface.surface_frame().window_state.focused, Some(true));
    assert_eq!(
        surface.surface_frame().window_state.application_active,
        Some(true)
    );
    assert_eq!(surface.surface_frame().window_state.occluded, Some(true));
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
    for result in [&focused, &active, &occluded] {
        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(
            result.diagnostics.route_policy,
            UiInputRoutePolicy::DefaultAction
        );
        assert_eq!(
            result.diagnostics.handled_phase.as_deref(),
            Some(UiDispatchPhase::DefaultAction.as_str())
        );
    }
    assert!(focused
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_focus_gained"));
    assert!(active
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_application_active"));
    assert!(occluded
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_occluded"));
}

#[test]
fn window_input_pump_batch_preserves_order_and_non_client_area_keeps_tooltip() {
    let mut surface = route_surface();
    open_popup(&mut surface, "menu.file", UiNodeId::new(2));
    show_tooltip(&mut surface, "status.hint", UiNodeId::new(3));

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push(UiWindowInputPumpEvent::Input(raw_mouse_motion_event(
        -3.5, 2.25,
    )));
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::window_action(
            window_metadata(9, false),
            UiWindowAction::ClickedNonClientArea,
        ),
    ));

    let results = dispatch_window_input_pump_batch(&mut surface, batch).unwrap();

    assert_eq!(results.len(), 2);
    assert!(matches!(
        results[0].event,
        UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
            delta_x: -3.5,
            delta_y: 2.25,
            ..
        })
    ));
    assert_eq!(
        results[0].diagnostics.route_policy,
        UiInputRoutePolicy::Unrouted
    );
    assert!(results[0]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "raw_mouse_motion"));

    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.input.tooltip.is_some());
    assert!(matches!(
        results[1].applied_effects[0].effect,
        UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::PopupStack,
            reason: UiTransientDismissalReason::WindowAction,
        }
    ));
    assert_eq!(results[1].diagnostics.route_target, Some(UiNodeId::new(2)));
    assert!(results[1]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_transient_dismissal"));
}

#[test]
fn window_input_pump_cursor_move_dispatches_unified_pointer_hover_route() {
    let mut surface = route_surface_with_hover_bindings();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(11, true),
            UiWindowEventKind::CursorMoved {
                position: UiPoint::new(20.0, 60.0),
                delta: Some(UiPoint::new(1.0, 2.0)),
            },
        )),
    )
    .unwrap();

    let UiInputEvent::Pointer(pointer) = &result.event else {
        panic!("expected cursor move to normalize into pointer input");
    };
    assert_eq!(
        pointer.metadata.window_id,
        Some(UiWindowId::new("main-window"))
    );
    assert!(pointer.metadata.synthetic);
    assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
    assert_eq!(pointer.event.point, UiPoint::new(20.0, 60.0));
    assert_eq!(pointer.precise_scroll, None);
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(3));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Hover { hovered: true }
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_normalized_input"));
}

#[test]
fn window_input_pump_cursor_left_replays_pointer_cancel_and_clears_hover() {
    let mut surface = route_surface_with_hover_bindings();

    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(12, true),
            UiWindowEventKind::CursorMoved {
                position: UiPoint::new(20.0, 60.0),
                delta: None,
            },
        )),
    )
    .unwrap();

    assert_eq!(
        surface.input.last_cursor_point(),
        Some(UiPoint::new(20.0, 60.0))
    );
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert!(surface
        .component_state(UiNodeId::new(3))
        .is_some_and(|state| state.flags.hovered));

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(13, true),
            UiWindowEventKind::CursorLeft,
        )),
    )
    .unwrap();

    let UiInputEvent::Pointer(pointer) = &result.event else {
        panic!("expected cursor leave to normalize into pointer cancel");
    };
    assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
    assert_eq!(pointer.event.point, UiPoint::new(20.0, 60.0));
    assert!(pointer.metadata.synthetic);
    assert_eq!(surface.input.last_cursor_point(), None);
    assert!(surface.focus.hovered.is_empty());
    assert!(!surface
        .component_state(UiNodeId::new(3))
        .is_some_and(|state| state.flags.hovered));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(3)
            && matches!(&event.event, UiComponentEvent::Hover { hovered: false })
    }));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_pointer_cancel"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_hover_cleared"));
}

#[test]
fn window_input_pump_touch_move_does_not_replace_last_mouse_cursor_point() {
    let mut surface = route_surface_with_hover_bindings();

    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(14, true),
            UiWindowEventKind::CursorMoved {
                position: UiPoint::new(20.0, 60.0),
                delta: None,
            },
        )),
    )
    .unwrap();

    let touch_input = UiWindowPlatformInputEvent::pointer(
        UiWindowInputContext::from_window_metadata(&window_metadata(15, true))
            .with_pointer_source(UiPointerSource::Touch),
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(70.0, 70.0)),
        None,
    )
    .normalize();
    dispatch_window_input_pump_event(&mut surface, UiWindowInputPumpEvent::Input(touch_input))
        .unwrap();

    assert_eq!(
        surface.input.last_cursor_point(),
        Some(UiPoint::new(20.0, 60.0))
    );

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(16, true),
            UiWindowEventKind::CursorLeft,
        )),
    )
    .unwrap();

    let UiInputEvent::Pointer(pointer) = &result.event else {
        panic!("expected cursor leave to normalize into pointer cancel");
    };
    assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
    assert_eq!(pointer.event.point, UiPoint::new(20.0, 60.0));
}

#[test]
fn window_input_pump_closed_without_cursor_point_clears_hover_without_fake_pointer_cancel() {
    let mut surface = route_surface_with_hover_bindings();
    surface.focus.hovered = vec![UiNodeId::new(3), UiNodeId::new(1)];
    let _ = surface.component_states.set_hovered(UiNodeId::new(3), true);

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(17, true),
            UiWindowEventKind::Closed,
        )),
    )
    .unwrap();

    assert!(surface.window_state.closed);
    assert!(surface.surface_frame().window_state.closed);
    assert!(matches!(
        &result.event,
        UiInputEvent::Popup(UiPopupInputEvent {
            kind: UiPopupInputEventKind::Dismissed,
            popup_id,
            ..
        }) if popup_id == "window.transient"
    ));
    assert!(surface.focus.hovered.is_empty());
    assert_eq!(surface.input.last_cursor_point(), None);
    assert!(!surface
        .component_state(UiNodeId::new(3))
        .is_some_and(|state| state.flags.hovered));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(3)
            && matches!(&event.event, UiComponentEvent::Hover { hovered: false })
    }));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_closed"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_pointer_cancel_missing_point"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_hover_cleared"));
}

#[test]
fn window_input_pump_retains_close_request_without_closing_the_surface() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::window_close(window_metadata(26, false))),
    )
    .unwrap();

    assert!(surface.window_state.close_requested);
    assert!(surface.surface_frame().window_state.close_requested);
    assert!(!surface.window_state.closed);
    assert!(!surface.window_state.destroyed);
    assert_eq!(surface.tree.roots, vec![UiNodeId::new(1)]);
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_close_requested"));
}

#[test]
fn window_input_pump_destroyed_retains_lifecycle_fact_and_clears_hover() {
    let mut surface = route_surface_with_hover_bindings();
    surface.focus.hovered = vec![UiNodeId::new(3), UiNodeId::new(1)];
    let _ = surface.component_states.set_hovered(UiNodeId::new(3), true);

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(27, true),
            UiWindowEventKind::Destroyed,
        )),
    )
    .unwrap();

    assert!(surface.window_state.destroyed);
    assert!(surface.surface_frame().window_state.destroyed);
    assert!(surface.focus.hovered.is_empty());
    assert_eq!(surface.input.last_cursor_point(), None);
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(3)
            && matches!(&event.event, UiComponentEvent::Hover { hovered: false })
    }));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_destroyed"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_pointer_cancel_missing_point"));
}

#[test]
fn window_input_pump_resize_updates_frame_metrics_and_layout_dirty_domains() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();
    let metrics = UiWindowMetrics::new(
        UiSize::new(320.0, 180.0),
        UiWindowPixelSize::new(640, 360),
        2.0,
    );

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
            window_metadata(18, false),
            metrics,
        )),
    )
    .unwrap();

    assert_eq!(surface.window_state.metrics, Some(metrics));
    assert_eq!(surface.surface_frame().window_state.metrics, Some(metrics));
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some(UiDispatchPhase::DefaultAction.as_str())
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_layout_metrics_dirty"));
}

#[test]
fn window_input_pump_scale_factor_updates_retained_metrics_without_losing_size() {
    let mut surface = route_surface();
    let metrics = UiWindowMetrics::new(
        UiSize::new(480.0, 270.0),
        UiWindowPixelSize::new(960, 540),
        2.0,
    );
    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
            window_metadata(19, false),
            metrics,
        )),
    )
    .unwrap();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(20, false),
            UiWindowEventKind::ScaleFactorChanged { scale_factor: 1.5 },
        )),
    )
    .unwrap();

    assert_eq!(
        surface.window_state.metrics,
        Some(UiWindowMetrics::new(
            metrics.logical_size,
            metrics.physical_size,
            1.5,
        ))
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_scale_factor_updated"));
}

#[test]
fn window_input_pump_move_updates_position_without_dirty_domains() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::moved_window(
            window_metadata(21, false),
            UiWindowPixelPosition::new(44, 88),
        )),
    )
    .unwrap();

    assert_eq!(
        surface.window_state.position,
        Some(UiWindowPixelPosition::new(44, 88))
    );
    assert_eq!(
        surface.surface_frame().window_state.position,
        Some(UiWindowPixelPosition::new(44, 88))
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_position_updated"));
}

#[test]
fn window_input_pump_redraw_request_marks_render_dirty_only() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::request_redraw(
            window_metadata(22, false),
            UiWindowRedrawReason::Animation,
        )),
    )
    .unwrap();

    assert!(surface.window_state.redraw_requested);
    assert_eq!(surface.window_state.redraw_request_count, 1);
    assert_eq!(
        surface.window_state.last_redraw_reason,
        Some(UiWindowRedrawReason::Animation)
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_redraw_requested"));
}

fn route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.window_input_pump"));
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

fn route_surface_with_hover_bindings() -> UiSurface {
    let mut surface = route_surface();
    for (node_id, control_id) in [
        (UiNodeId::new(2), "FirstHover"),
        (UiNodeId::new(3), "SecondHover"),
    ] {
        let target = surface.tree.nodes.get_mut(&node_id).unwrap();
        target.template_metadata = Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            bindings: vec![binding(&format!("{control_id}/Hover"), UiEventKind::Hover)],
            ..Default::default()
        });
    }
    surface.rebuild();
    surface
}

fn dispatch_window_input_pump_event(
    surface: &mut UiSurface,
    event: UiWindowInputPumpEvent,
) -> Result<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult, UiTreeError> {
    let mut manager = UiInputManager::default();
    surface.dispatch_window_input_pump_event(&mut manager, event)
}

fn dispatch_window_input_pump_batch(
    surface: &mut UiSurface,
    batch: UiWindowInputPumpBatch,
) -> Result<Vec<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult>, UiTreeError> {
    let mut manager = UiInputManager::default();
    surface
        .dispatch_window_input_pump_batch(&mut manager, batch)
        .map(|outcome| outcome.results)
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
    UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1))
}

fn window_metadata(sequence: u64, synthetic: bool) -> UiWindowEventMetadata {
    UiWindowEventMetadata::for_window(
        UiWindowId::new("main-window"),
        UiInputTimestamp::from_micros(100 + sequence),
        UiInputSequence::new(sequence),
    )
    .synthetic(synthetic)
}

fn popup_event(kind: UiPopupInputEventKind, popup_id: &str, owner: UiNodeId) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner: Some(owner),
        anchor: Some(UiPoint::new(8.0, 12.0)),
    })
}

fn tooltip_event(
    kind: UiTooltipTimerInputEventKind,
    tooltip_id: &str,
    owner: UiNodeId,
) -> UiInputEvent {
    UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
        metadata: input_metadata(),
        kind,
        tooltip_id: tooltip_id.to_string(),
        owner: Some(owner),
    })
}

fn raw_mouse_motion_event(delta_x: f32, delta_y: f32) -> UiInputEvent {
    UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
        metadata: input_metadata(),
        delta_x,
        delta_y,
    })
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

fn open_popup(surface: &mut UiSurface, popup_id: &str, owner: UiNodeId) {
    surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, popup_id, owner),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: popup_id.to_string(),
            owner: Some(owner),
            anchor: Some(UiPoint::new(8.0, 12.0)),
        }),
    );
}

fn show_tooltip(surface: &mut UiSurface, tooltip_id: &str, owner: UiNodeId) {
    surface.apply_dispatch_reply(
        tooltip_event(UiTooltipTimerInputEventKind::Elapsed, tooltip_id, owner),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Show,
            tooltip_id: tooltip_id.to_string(),
            owner: Some(owner),
        }),
    );
}
