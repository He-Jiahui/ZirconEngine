use zircon_runtime_interface::ui::{
    dispatch::{
        UiComponentEventReport, UiDispatchPhase, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent, UiPopupInputEvent, UiPopupInputEventKind,
    },
    surface::UiSurfaceWindowState,
    tree::UiDirtyFlags,
    tree::UiTreeError,
    window::{
        UiWindowEvent, UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpBatch,
        UiWindowInputPumpEvent, UiWindowRedrawReason,
    },
};

use super::super::surface::UiSurface;
use super::{apply_dispatch_reply, dispatch_input_event};
use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};

pub(crate) fn dispatch_window_input_pump_event(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    navigation_dispatcher: &UiNavigationDispatcher,
    event: UiWindowInputPumpEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    match event {
        UiWindowInputPumpEvent::Input(input) => {
            dispatch_input_event(surface, pointer_dispatcher, navigation_dispatcher, input)
        }
        UiWindowInputPumpEvent::Window(window_event) => dispatch_window_event(
            surface,
            pointer_dispatcher,
            navigation_dispatcher,
            window_event,
        ),
    }
}

pub(crate) fn dispatch_window_input_pump_batch(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    navigation_dispatcher: &UiNavigationDispatcher,
    batch: UiWindowInputPumpBatch,
) -> Result<Vec<UiInputDispatchResult>, UiTreeError> {
    batch
        .events
        .into_iter()
        .map(|event| {
            dispatch_window_input_pump_event(
                surface,
                pointer_dispatcher,
                navigation_dispatcher,
                event,
            )
        })
        .collect()
}

fn dispatch_window_event(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    navigation_dispatcher: &UiNavigationDispatcher,
    event: UiWindowEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let retained_note = apply_window_lifecycle_effect(surface, &event);

    if let Some(input) = event.normalized_cursor_move_input() {
        let mut result =
            dispatch_input_event(surface, pointer_dispatcher, navigation_dispatcher, input)?;
        mark_optional_window_event_result(&mut result, retained_note);
        mark_window_event_result(&mut result, "window_normalized_input");
        return Ok(result);
    }

    if let Some(point) = surface.input.last_cursor_point() {
        if let Some(input) = event.normalized_pointer_cancel_input(point) {
            let mut result =
                dispatch_input_event(surface, pointer_dispatcher, navigation_dispatcher, input)?;
            append_window_hover_clear_result(surface, &event, &mut result)?;
            mark_optional_window_event_result(&mut result, retained_note);
            mark_window_event_result(&mut result, "window_pointer_cancel");
            return Ok(result);
        }
    }

    if event.impact().clears_hover {
        let synthetic_input = window_event_transient_input(&event);
        let mut result = UiInputDispatchResult::new(synthetic_input, UiDispatchReply::unhandled());
        append_fallback_pointer_interaction_clear(surface, &mut result)?;
        mark_optional_window_event_result(&mut result, retained_note);
        mark_window_event_result(&mut result, "window_pointer_cancel_missing_point");
        return Ok(result);
    }

    let synthetic_input = window_event_transient_input(&event);
    if let Some(effect) = event.transient_dismissal_effect() {
        let mut result = apply_dispatch_reply(
            surface,
            synthetic_input,
            UiDispatchReply::handled()
                .in_phase(UiDispatchPhase::DefaultAction)
                .with_effect(effect),
        );
        mark_optional_window_event_result(&mut result, retained_note);
        mark_window_event_result(&mut result, "window_transient_dismissal");
        return Ok(result);
    }

    if let Some(note) = apply_window_surface_effect(surface, &event)? {
        let mut result = handled_window_event_result(synthetic_input);
        mark_optional_window_event_result(&mut result, retained_note);
        mark_window_event_result(&mut result, note);
        return Ok(result);
    }

    if let Some(note) = retained_note {
        let mut result = handled_window_event_result(synthetic_input);
        mark_window_event_result(&mut result, note);
        return Ok(result);
    }

    let mut result = UiInputDispatchResult::new(synthetic_input, UiDispatchReply::unhandled());
    mark_optional_window_event_result(&mut result, retained_note);
    mark_window_event_result(&mut result, "window_event_no_input_effect");
    Ok(result)
}

fn handled_window_event_result(event: UiInputEvent) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(
        event,
        UiDispatchReply::handled().in_phase(UiDispatchPhase::DefaultAction),
    );
    result.diagnostics.routed = true;
    result.diagnostics.route_policy =
        zircon_runtime_interface::ui::dispatch::UiInputRoutePolicy::DefaultAction;
    result.diagnostics.handled_phase = Some(UiDispatchPhase::DefaultAction.as_str().to_string());
    result
}

fn apply_window_surface_effect(
    surface: &mut UiSurface,
    event: &UiWindowEvent,
) -> Result<Option<&'static str>, UiTreeError> {
    match &event.kind {
        UiWindowEventKind::Created { metrics } | UiWindowEventKind::Resized { metrics } => {
            surface.window_state.metrics = Some(*metrics);
            mark_roots_dirty(surface, layout_metrics_dirty())?;
            Ok(Some("window_layout_metrics_dirty"))
        }
        UiWindowEventKind::ScaleFactorChanged { scale_factor }
        | UiWindowEventKind::BackendScaleFactorChanged { scale_factor } => {
            update_scale_factor(&mut surface.window_state, *scale_factor);
            mark_roots_dirty(surface, layout_metrics_dirty())?;
            Ok(Some("window_scale_factor_updated"))
        }
        UiWindowEventKind::Moved { position } => {
            surface.window_state.position = Some(*position);
            Ok(Some("window_position_updated"))
        }
        UiWindowEventKind::RequestRedraw { reason } => {
            record_redraw_request(&mut surface.window_state, *reason);
            mark_roots_dirty(surface, render_dirty())?;
            Ok(Some("window_redraw_requested"))
        }
        _ => Ok(None),
    }
}

fn apply_window_lifecycle_effect(
    surface: &mut UiSurface,
    event: &UiWindowEvent,
) -> Option<&'static str> {
    match &event.kind {
        UiWindowEventKind::Focused { focused } => {
            surface.window_state.focused = Some(*focused);
            Some(if *focused {
                "window_focus_gained"
            } else {
                "window_focus_lost"
            })
        }
        UiWindowEventKind::ApplicationActivation { is_active } => {
            surface.window_state.application_active = Some(*is_active);
            Some(if *is_active {
                "window_application_active"
            } else {
                "window_application_inactive"
            })
        }
        UiWindowEventKind::Occluded { occluded } => {
            surface.window_state.occluded = Some(*occluded);
            Some(if *occluded {
                "window_occluded"
            } else {
                "window_unoccluded"
            })
        }
        UiWindowEventKind::CloseRequested => {
            surface.window_state.close_requested = true;
            Some("window_close_requested")
        }
        UiWindowEventKind::Closed => {
            surface.window_state.closed = true;
            Some("window_closed")
        }
        UiWindowEventKind::Destroyed => {
            surface.window_state.destroyed = true;
            Some("window_destroyed")
        }
        _ => None,
    }
}

fn update_scale_factor(window_state: &mut UiSurfaceWindowState, scale_factor: f64) {
    let mut metrics = window_state.metrics.unwrap_or_default();
    metrics.scale_factor = scale_factor;
    window_state.metrics = Some(metrics);
}

fn record_redraw_request(window_state: &mut UiSurfaceWindowState, reason: UiWindowRedrawReason) {
    window_state.redraw_requested = true;
    window_state.redraw_request_count = window_state.redraw_request_count.saturating_add(1);
    window_state.last_redraw_reason = Some(reason);
}

fn mark_roots_dirty(surface: &mut UiSurface, dirty: UiDirtyFlags) -> Result<(), UiTreeError> {
    for root_id in surface.tree.roots.clone() {
        surface.mark_node_dirty(root_id, dirty)?;
    }
    Ok(())
}

fn layout_metrics_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        ..Default::default()
    }
}

fn render_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        render: true,
        ..Default::default()
    }
}

fn mark_window_event_result(result: &mut UiInputDispatchResult, note: &str) {
    result.diagnostics.notes.push("window_event".to_string());
    result
        .diagnostics
        .notes
        .push("window_input_pump".to_string());
    result.diagnostics.notes.push(note.to_string());
}

fn mark_optional_window_event_result(result: &mut UiInputDispatchResult, note: Option<&str>) {
    if let Some(note) = note {
        result.diagnostics.notes.push(note.to_string());
    }
}

fn append_window_hover_clear_result(
    surface: &mut UiSurface,
    event: &UiWindowEvent,
    result: &mut UiInputDispatchResult,
) -> Result<(), UiTreeError> {
    if !event.impact().clears_hover {
        return Ok(());
    }

    append_component_events(
        result,
        surface.clear_hovered_input_path()?,
        "window_hover_cleared",
    );
    Ok(())
}

fn append_fallback_pointer_interaction_clear(
    surface: &mut UiSurface,
    result: &mut UiInputDispatchResult,
) -> Result<(), UiTreeError> {
    append_component_events(
        result,
        surface.clear_pointer_interaction_without_route()?,
        "window_hover_cleared",
    );
    Ok(())
}

fn append_component_events(
    result: &mut UiInputDispatchResult,
    component_events: Vec<UiComponentEventReport>,
    note: &str,
) {
    if component_events.is_empty() {
        return;
    }

    result.component_events.extend(component_events);
    result.diagnostics.notes.push(note.to_string());
}

fn window_event_transient_input(event: &UiWindowEvent) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: UiWindowInputContext::from_window_metadata(&event.metadata).metadata,
        kind: UiPopupInputEventKind::Dismissed,
        popup_id: "window.transient".to_string(),
        owner: None,
        anchor: None,
    })
}
