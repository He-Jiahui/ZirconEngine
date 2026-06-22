use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiDragDropInputEventKind, UiInputDispatchResult, UiInputEvent,
        UiInputRoutePolicy, UiInputRouteTrace, UiPointerInputEvent,
    },
    event_ui::UiNodeId,
    surface::{UiNavigationRoute, UiPointerEventKind, UiPointerRoute},
};

use crate::ui::dispatch::{route_policy_uses_stage, UiInputRouteStage};
use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::{super::arranged_focus_path, super::surface::UiSurface, state::UiSurfaceInputState};

pub(super) fn route_policy_for_input_event(
    input: &UiSurfaceInputState,
    event: &UiInputEvent,
) -> UiInputRoutePolicy {
    match event {
        UiInputEvent::Pointer(pointer) => match pointer.event.kind {
            UiPointerEventKind::Up if pointer_event_has_capture(input, pointer) => {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Down | UiPointerEventKind::Up | UiPointerEventKind::Scroll => {
                UiInputRoutePolicy::Bubble
            }
            UiPointerEventKind::Move if pointer_event_has_capture(input, pointer) => {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Cancel if pointer_event_has_capture(input, pointer) => {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Move | UiPointerEventKind::Cancel => UiInputRoutePolicy::Direct,
        },
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_) => {
            UiInputRoutePolicy::FocusPath
        }
        UiInputEvent::Navigation(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::Analog(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::MouseMotion(_) => UiInputRoutePolicy::Unrouted,
        UiInputEvent::DragDrop(drag_drop) => match drag_drop.kind {
            UiDragDropInputEventKind::Begin => UiInputRoutePolicy::Direct,
            UiDragDropInputEventKind::Enter
            | UiDragDropInputEventKind::Over
            | UiDragDropInputEventKind::Leave
            | UiDragDropInputEventKind::Drop => UiInputRoutePolicy::Bubble,
            UiDragDropInputEventKind::End => UiInputRoutePolicy::PointerCapture,
        },
        UiInputEvent::Popup(_)
        | UiInputEvent::TooltipTimer(_)
        | UiInputEvent::TypeaheadTimer(_)
        | UiInputEvent::SubmenuHoverTimer(_)
        | UiInputEvent::ToastTimer(_)
        | UiInputEvent::Accessibility(_) => UiInputRoutePolicy::DefaultAction,
    }
}

pub(super) fn annotate_route_policy(
    surface: &UiSurface,
    event: &UiInputEvent,
    result: &mut UiInputDispatchResult,
) {
    let released_capture_target = pointer_capture_release_target(event, result);
    result.diagnostics.route_policy = if released_capture_target.is_some() {
        UiInputRoutePolicy::PointerCapture
    } else {
        route_policy_for_input_event(&surface.input, event)
    };
    populate_generic_route_trace(surface, event, result, released_capture_target);
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
    capture_override: Option<UiNodeId>,
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
    let capture_target = capture_target_for_event(surface, event, capture_override);
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

fn pointer_event_has_capture(input: &UiSurfaceInputState, pointer: &UiPointerInputEvent) -> bool {
    pointer
        .metadata
        .pointer_id
        .is_some_and(|pointer_id| input.pointer_capture_owner(pointer_id).is_some())
        || (input.captured_pointer_id.is_some()
            && input.captured_pointer_id == pointer.metadata.pointer_id)
}

fn pointer_capture_release_target(
    event: &UiInputEvent,
    result: &UiInputDispatchResult,
) -> Option<UiNodeId> {
    let UiInputEvent::Pointer(pointer) = event else {
        return None;
    };
    if !matches!(
        pointer.event.kind,
        UiPointerEventKind::Up | UiPointerEventKind::Cancel
    ) {
        return None;
    }
    let pointer_id = pointer.metadata.pointer_id.unwrap_or_default();
    result
        .applied_effects
        .iter()
        .find_map(|applied| match &applied.effect {
            UiDispatchEffect::ReleasePointerCapture {
                target,
                pointer_id: released_pointer_id,
                ..
            } if *released_pointer_id == pointer_id => Some(*target),
            _ => None,
        })
}

fn capture_target_for_event(
    surface: &UiSurface,
    event: &UiInputEvent,
    capture_override: Option<UiNodeId>,
) -> Option<UiNodeId> {
    capture_override
        .or_else(|| match event {
            UiInputEvent::Pointer(pointer) => pointer
                .metadata
                .pointer_id
                .and_then(|pointer_id| surface.input.pointer_capture_owner(pointer_id)),
            _ => None,
        })
        .or(surface.focus.captured)
}

fn event_owner(event: &UiInputEvent) -> Option<UiNodeId> {
    match event {
        UiInputEvent::Popup(popup) => popup.owner,
        UiInputEvent::TooltipTimer(tooltip) => tooltip.owner,
        UiInputEvent::TypeaheadTimer(typeahead) => Some(typeahead.target),
        UiInputEvent::SubmenuHoverTimer(submenu_hover) => Some(submenu_hover.target),
        UiInputEvent::ToastTimer(toast) => Some(toast.target),
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
    if route_policy_uses_stage(policy, UiInputRouteStage::PointerCapture) {
        return capture_target.or(target);
    }

    let direct_only = route_policy_uses_stage(policy, UiInputRouteStage::DirectTarget)
        && !route_policy_uses_stage(policy, UiInputRouteStage::PreviewTunnel)
        && !route_policy_uses_stage(policy, UiInputRouteStage::BubblePath);
    let default_action = route_policy_uses_stage(policy, UiInputRouteStage::DefaultAction);
    if direct_only || default_action {
        return target;
    }

    None
}

fn preview_tunnel_for_bubble(bubble_path: &[UiNodeId]) -> Vec<UiNodeId> {
    bubble_path.iter().rev().copied().collect()
}

fn focused_route(surface: &UiSurface, focused: Option<UiNodeId>) -> Vec<UiNodeId> {
    arranged_focus_path(&surface.arranged_tree, focused.or(surface.focus.focused)).bubble_route
}

fn popup_stack(surface: &UiSurface) -> Vec<String> {
    surface
        .input
        .popup_stack
        .iter()
        .map(|popup| popup.popup_id.clone())
        .collect()
}
