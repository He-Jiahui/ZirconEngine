use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiDragDropInputEventKind, UiInputDiagnosticsMode, UiInputDispatchResult,
        UiInputEvent, UiInputRoutePolicy, UiInputRouteTrace, UiPointerId, UiPointerInputEvent,
        UiPointerRoutingReceipt, UiPointerSource,
    },
    event_ui::UiNodeId,
    surface::{UiNavigationRoute, UiPointerEventKind, UiPointerRoute},
};

use crate::ui::dispatch::{route_policy_uses_stage, UiInputRouteStage};
use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::{
    super::{arranged_node_indexed, surface::UiSurface},
    diagnostics_budget::{bounded_node_path, bounded_popup_stack, MAX_ROUTE_NODES_PER_PATH},
    state::UiSurfaceInputState,
};

pub(super) fn route_policy_for_input_event(
    input: &UiSurfaceInputState,
    event: &UiInputEvent,
) -> UiInputRoutePolicy {
    match event {
        UiInputEvent::Pointer(pointer) => pointer_route_policy(input, pointer),
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_) => {
            UiInputRoutePolicy::FocusPath
        }
        UiInputEvent::Navigation(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::Analog(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::MouseMotion(_) => UiInputRoutePolicy::Unrouted,
        UiInputEvent::Clipboard(_) => UiInputRoutePolicy::DefaultAction,
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
    let released_capture_target = annotate_route_policy_fields(surface, event, result);
    populate_generic_route_trace(surface, event, result, released_capture_target);
}

fn annotate_route_policy_fields(
    surface: &UiSurface,
    event: &UiInputEvent,
    result: &mut UiInputDispatchResult,
) -> Option<UiNodeId> {
    let released_capture_target = pointer_capture_release_target(event, result);
    result.diagnostics.route_policy = if released_capture_target.is_some() {
        UiInputRoutePolicy::PointerCapture
    } else {
        route_policy_for_input_event(&surface.input, event)
    };
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
    released_capture_target
}

pub(super) fn annotate_pointer_route_trace(
    surface: &UiSurface,
    route: UiPointerRoute,
    pointer_source: UiPointerSource,
    pointer_id: Option<UiPointerId>,
    diagnostics_mode: UiInputDiagnosticsMode,
    result: &mut UiInputDispatchResult,
) {
    let released_capture_target =
        pointer_capture_release_target_for_pointer(route.kind, pointer_id, result);
    result.diagnostics.route_policy = if released_capture_target.is_some() {
        UiInputRoutePolicy::PointerCapture
    } else {
        pointer_route_policy_for_parts(&surface.input, route.kind, pointer_id)
    };
    let capture_terminal = is_capture_terminal_pointer_route(&route);
    if capture_terminal {
        result.diagnostics.route_policy = UiInputRoutePolicy::PointerCapture;
    }
    let capture_target = route.captured.or(surface.focus.captured);
    let direct_target = direct_target_for_policy(
        result.diagnostics.route_policy,
        route.target,
        capture_target,
    );
    let UiPointerRoute {
        target,
        hit_path,
        routing_path,
        focused,
        root_targets,
        ..
    } = route;
    let receipt = UiPointerRoutingReceipt {
        route_target: target,
        capture_target,
        physical_hit_path: hit_path,
        dispatch_path: routing_path,
    };
    result.pointer_routing = Some(receipt);
    if !diagnostics_mode.captures_full_trace() {
        return;
    }

    annotate_pointer_source(pointer_source, result);
    let receipt = result
        .pointer_routing
        .as_ref()
        .expect("pointer routing receipt was assigned before trace projection");
    let diagnostics = &mut result.diagnostics;
    let preview_tunnel = bounded_node_path(
        receipt.dispatch_root_to_leaf().iter().copied(),
        &mut diagnostics.truncation,
    );
    let bubble_path =
        bounded_node_path(receipt.dispatch_bubble_route(), &mut diagnostics.truncation);
    let focus_path = focused_route_bounded(surface, focused, &mut diagnostics.truncation);
    let root_targets = bounded_node_path(root_targets.into_iter(), &mut diagnostics.truncation);
    let popup_stack = bounded_popup_stack(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str()),
        &mut diagnostics.truncation,
    );
    diagnostics.route_trace = UiInputRouteTrace {
        preview_tunnel,
        direct_target,
        target,
        bubble_path,
        focus_path,
        capture_target,
        root_targets,
        popup_stack,
    };
}

pub(super) fn annotate_navigation_route_trace(
    surface: &UiSurface,
    route: UiNavigationRoute,
    result: &mut UiInputDispatchResult,
) {
    result.diagnostics.route_policy = UiInputRoutePolicy::FocusPath;
    let preview_tunnel = preview_tunnel_for_bubble(&route.bubbled);
    let focus_path = route.bubbled.clone();
    result.diagnostics.route_trace = UiInputRouteTrace {
        preview_tunnel,
        direct_target: None,
        target: route.target,
        bubble_path: route.bubbled,
        focus_path,
        capture_target: surface.focus.captured,
        root_targets: route.root_targets,
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

fn pointer_id_has_capture(input: &UiSurfaceInputState, pointer_id: Option<UiPointerId>) -> bool {
    pointer_id.is_some_and(|pointer_id| input.pointer_capture_owner(pointer_id).is_some())
}

fn pointer_route_policy(
    input: &UiSurfaceInputState,
    pointer: &UiPointerInputEvent,
) -> UiInputRoutePolicy {
    pointer_route_policy_for_parts(input, pointer.event.kind, pointer.metadata.pointer_id)
}

fn pointer_route_policy_for_parts(
    input: &UiSurfaceInputState,
    kind: UiPointerEventKind,
    pointer_id: Option<UiPointerId>,
) -> UiInputRoutePolicy {
    match kind {
        UiPointerEventKind::Up if pointer_id_has_capture(input, pointer_id) => {
            UiInputRoutePolicy::PointerCapture
        }
        UiPointerEventKind::Down | UiPointerEventKind::Up | UiPointerEventKind::Scroll => {
            UiInputRoutePolicy::Bubble
        }
        UiPointerEventKind::Move if pointer_id_has_capture(input, pointer_id) => {
            UiInputRoutePolicy::PointerCapture
        }
        UiPointerEventKind::Cancel if pointer_id_has_capture(input, pointer_id) => {
            UiInputRoutePolicy::PointerCapture
        }
        UiPointerEventKind::Move | UiPointerEventKind::Cancel => UiInputRoutePolicy::Direct,
    }
}

fn annotate_pointer_source(pointer_source: UiPointerSource, result: &mut UiInputDispatchResult) {
    let source = match pointer_source {
        UiPointerSource::Mouse => "pointer_source=Mouse",
        UiPointerSource::Touch => "pointer_source=Touch",
        UiPointerSource::Pen => "pointer_source=Pen",
        UiPointerSource::Unknown => "pointer_source=Unknown",
    };
    result.diagnostics.notes.push(source.to_string());
    if pointer_source.is_touch_like() {
        result
            .diagnostics
            .notes
            .push("touch_like_pointer".to_string());
    }
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

fn pointer_capture_release_target_for_pointer(
    kind: UiPointerEventKind,
    pointer_id: Option<UiPointerId>,
    result: &UiInputDispatchResult,
) -> Option<UiNodeId> {
    if !matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel) {
        return None;
    }
    let pointer_id = pointer_id.unwrap_or_default();
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
        UiInputEvent::Clipboard(clipboard) => Some(clipboard.owner),
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

fn focused_route_bounded(
    surface: &UiSurface,
    focused: Option<UiNodeId>,
    truncation: &mut zircon_runtime_interface::ui::dispatch::UiInputDiagnosticsTruncationReceipt,
) -> Vec<UiNodeId> {
    let Some(focused) = focused.or(surface.focus.focused) else {
        return Vec::new();
    };
    let mut route = Vec::with_capacity(MAX_ROUTE_NODES_PER_PATH.min(16));
    let mut current = Some(focused);
    let mut dropped = 0_u64;
    while let Some(node_id) = current {
        let Ok(node) = arranged_node_indexed(
            &surface.arranged_tree,
            &surface.arranged_node_indices,
            node_id,
        ) else {
            return Vec::new();
        };
        if route.len() < MAX_ROUTE_NODES_PER_PATH {
            route.push(node_id);
        } else {
            dropped = dropped.saturating_add(1);
        }
        current = node.parent;
    }
    truncation.route_nodes_dropped = truncation.route_nodes_dropped.saturating_add(dropped);
    route
}

fn popup_stack(surface: &UiSurface) -> Vec<String> {
    surface
        .input
        .popup_stack
        .iter()
        .map(|popup| popup.popup_id.clone())
        .collect()
}
