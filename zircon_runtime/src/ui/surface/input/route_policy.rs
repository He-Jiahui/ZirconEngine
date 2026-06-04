use zircon_runtime_interface::ui::{
    dispatch::{
        UiDragDropInputEventKind, UiInputDispatchResult, UiInputEvent, UiInputRoutePolicy,
        UiInputRouteTrace,
    },
    event_ui::UiNodeId,
    surface::{UiNavigationRoute, UiPointerEventKind, UiPointerRoute},
};

use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::{super::surface::UiSurface, state::UiSurfaceInputState};

pub(super) fn route_policy_for_input_event(
    input: &UiSurfaceInputState,
    event: &UiInputEvent,
) -> UiInputRoutePolicy {
    match event {
        UiInputEvent::Pointer(pointer) => match pointer.event.kind {
            UiPointerEventKind::Up
                if input.captured_pointer_id.is_some()
                    && input.captured_pointer_id == pointer.metadata.pointer_id =>
            {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Down | UiPointerEventKind::Up | UiPointerEventKind::Scroll => {
                UiInputRoutePolicy::Bubble
            }
            UiPointerEventKind::Move
                if input.captured_pointer_id.is_some()
                    && input.captured_pointer_id == pointer.metadata.pointer_id =>
            {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Cancel
                if input.captured_pointer_id.is_some()
                    && input.captured_pointer_id == pointer.metadata.pointer_id =>
            {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Move | UiPointerEventKind::Cancel => UiInputRoutePolicy::Direct,
        },
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_) => {
            UiInputRoutePolicy::FocusPath
        }
        UiInputEvent::Navigation(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::Analog(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::DragDrop(drag_drop) => match drag_drop.kind {
            UiDragDropInputEventKind::Begin => UiInputRoutePolicy::Direct,
            UiDragDropInputEventKind::Enter
            | UiDragDropInputEventKind::Over
            | UiDragDropInputEventKind::Leave
            | UiDragDropInputEventKind::Drop => UiInputRoutePolicy::Bubble,
            UiDragDropInputEventKind::End => UiInputRoutePolicy::PointerCapture,
        },
        UiInputEvent::Popup(_) | UiInputEvent::TooltipTimer(_) | UiInputEvent::Accessibility(_) => {
            UiInputRoutePolicy::DefaultAction
        }
    }
}

pub(super) fn annotate_route_policy(
    surface: &UiSurface,
    event: &UiInputEvent,
    result: &mut UiInputDispatchResult,
) {
    result.diagnostics.route_policy = route_policy_for_input_event(&surface.input, event);
    populate_generic_route_trace(surface, event, result);
    if let UiInputEvent::Pointer(pointer) = event {
        result.diagnostics.notes.push(format!(
            "pointer_source={:?}",
            pointer.metadata.pointer_source
        ));
        if pointer.metadata.pointer_source.is_touch_like() {
            result
                .diagnostics
                .notes
                .push("touch_like_pointer".to_string());
        }
    }
}

pub(super) fn annotate_pointer_route_trace(
    surface: &UiSurface,
    route: &UiPointerRoute,
    event: &UiInputEvent,
    result: &mut UiInputDispatchResult,
) {
    annotate_route_policy(surface, event, result);
    if is_capture_terminal_pointer_route(route) {
        result.diagnostics.route_policy = UiInputRoutePolicy::PointerCapture;
    }
    result.diagnostics.route_trace = UiInputRouteTrace {
        preview_tunnel: preview_tunnel_for_bubble(&route.bubbled),
        direct_target: direct_target_for_policy(
            result.diagnostics.route_policy,
            route.target,
            route.captured.or(surface.focus.captured),
        ),
        target: route.target,
        bubble_path: route.bubbled.clone(),
        focus_path: focused_route(surface, route.focused),
        capture_target: route.captured.or(surface.focus.captured),
        root_targets: route.root_targets.clone(),
        popup_stack: popup_stack(surface),
    };
}

pub(super) fn annotate_navigation_route_trace(
    surface: &UiSurface,
    route: &UiNavigationRoute,
    event: &UiInputEvent,
    result: &mut UiInputDispatchResult,
) {
    annotate_route_policy(surface, event, result);
    result.diagnostics.route_trace = UiInputRouteTrace {
        preview_tunnel: preview_tunnel_for_bubble(&route.bubbled),
        direct_target: None,
        target: route.target,
        bubble_path: route.bubbled.clone(),
        focus_path: route.bubbled.clone(),
        capture_target: surface.focus.captured,
        root_targets: route.root_targets.clone(),
        popup_stack: popup_stack(surface),
    };
}

fn populate_generic_route_trace(
    surface: &UiSurface,
    event: &UiInputEvent,
    result: &mut UiInputDispatchResult,
) {
    let target = event_owner(event).or(result.diagnostics.route_target);
    let bubble_path = target
        .and_then(|target| surface.tree.bubble_route(target).ok())
        .unwrap_or_default();
    let focus_path = surface.focused_route();
    let route_path = if bubble_path.is_empty() {
        focus_path.clone()
    } else {
        bubble_path.clone()
    };
    let capture_target = surface.focus.captured;
    result.diagnostics.route_trace = UiInputRouteTrace {
        preview_tunnel: preview_tunnel_for_bubble(&route_path),
        direct_target: direct_target_for_policy(
            result.diagnostics.route_policy,
            target,
            capture_target,
        ),
        target,
        bubble_path,
        focus_path,
        capture_target,
        root_targets: if target.is_none() {
            surface.tree.roots.clone()
        } else {
            Vec::new()
        },
        popup_stack: popup_stack(surface),
    };
}

fn event_owner(event: &UiInputEvent) -> Option<UiNodeId> {
    match event {
        UiInputEvent::Popup(popup) => popup.owner,
        UiInputEvent::TooltipTimer(tooltip) => tooltip.owner,
        UiInputEvent::Accessibility(accessibility) => Some(accessibility.request.target),
        _ => None,
    }
}

fn is_capture_terminal_pointer_route(route: &UiPointerRoute) -> bool {
    matches!(
        route.kind,
        UiPointerEventKind::Up | UiPointerEventKind::Cancel
    ) && route.captured.is_some()
}

fn direct_target_for_policy(
    policy: UiInputRoutePolicy,
    target: Option<UiNodeId>,
    capture_target: Option<UiNodeId>,
) -> Option<UiNodeId> {
    match policy {
        UiInputRoutePolicy::PointerCapture => capture_target.or(target),
        UiInputRoutePolicy::Direct | UiInputRoutePolicy::DefaultAction => target,
        UiInputRoutePolicy::Unrouted
        | UiInputRoutePolicy::PreviewTunnel
        | UiInputRoutePolicy::Bubble
        | UiInputRoutePolicy::FocusPath => None,
    }
}

fn preview_tunnel_for_bubble(bubble_path: &[UiNodeId]) -> Vec<UiNodeId> {
    bubble_path.iter().rev().copied().collect()
}

fn focused_route(surface: &UiSurface, focused: Option<UiNodeId>) -> Vec<UiNodeId> {
    focused
        .or(surface.focus.focused)
        .and_then(|focused| surface.tree.bubble_route(focused).ok())
        .unwrap_or_default()
}

fn popup_stack(surface: &UiSurface) -> Vec<String> {
    surface
        .input
        .popup_stack
        .iter()
        .map(|popup| popup.popup_id.clone())
        .collect()
}
