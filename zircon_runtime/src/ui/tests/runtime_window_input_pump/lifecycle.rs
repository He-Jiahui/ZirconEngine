use super::*;

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
