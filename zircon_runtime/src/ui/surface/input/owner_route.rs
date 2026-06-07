use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchDisposition, UiDispatchReply, UiInputDispatchResult, UiInputEvent},
    event_ui::UiNodeId,
    focus::UiFocusedInputKind,
};

use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::super::surface::UiSurface;
use super::is_valid_input_owner;

pub(super) fn owner_routed_result(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: Option<UiNodeId>,
    phase: &str,
) -> UiInputDispatchResult {
    let kind = focused_input_kind_for_event(&event);
    let valid_target = target.filter(|node_id| is_valid_input_owner(surface, *node_id));
    let reply = if valid_target.is_some() {
        UiDispatchReply::handled()
    } else {
        UiDispatchReply::unhandled()
    };
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = valid_target.is_some();
    result.diagnostics.route_target = valid_target;
    result.diagnostics.handled_phase = valid_target.map(|_| phase.to_string());
    if target.is_some() && valid_target.is_none() {
        result
            .diagnostics
            .notes
            .push("owner route rejected".to_string());
    }
    if let (Some(kind), Some(target)) = (kind, valid_target) {
        record_owner_focused_input(
            surface,
            kind,
            target,
            Some(target),
            result.reply.disposition != UiDispatchDisposition::Unhandled,
        );
    }
    result
}

pub(super) fn focused_input_kind_for_event(event: &UiInputEvent) -> Option<UiFocusedInputKind> {
    match event {
        UiInputEvent::Keyboard(_) => Some(UiFocusedInputKind::Keyboard),
        UiInputEvent::Text(_) => Some(UiFocusedInputKind::Text),
        UiInputEvent::Ime(_) => Some(UiFocusedInputKind::Ime),
        UiInputEvent::Navigation(_) => Some(UiFocusedInputKind::Navigation),
        UiInputEvent::Pointer(_) => Some(UiFocusedInputKind::Pointer),
        UiInputEvent::Analog(_)
        | UiInputEvent::MouseMotion(_)
        | UiInputEvent::DragDrop(_)
        | UiInputEvent::Popup(_)
        | UiInputEvent::TooltipTimer(_)
        | UiInputEvent::Accessibility(_) => None,
    }
}

pub(super) fn record_owner_focused_input(
    surface: &mut UiSurface,
    kind: UiFocusedInputKind,
    target: UiNodeId,
    handled_by: Option<UiNodeId>,
    accepted: bool,
) {
    let route = surface.tree.bubble_route(target).unwrap_or_default();
    surface.record_focused_input(kind, target, route, handled_by, accepted);
}
