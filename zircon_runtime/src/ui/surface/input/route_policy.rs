use zircon_runtime_interface::ui::{
    dispatch::{UiInputEvent, UiInputRoutePolicy},
    surface::UiPointerEventKind,
};

use super::state::UiSurfaceInputState;

pub(super) fn route_policy_for_input_event(
    input: &UiSurfaceInputState,
    event: &UiInputEvent,
) -> UiInputRoutePolicy {
    match event {
        UiInputEvent::Pointer(pointer) => match pointer.event.kind {
            UiPointerEventKind::Down | UiPointerEventKind::Up | UiPointerEventKind::Scroll => {
                UiInputRoutePolicy::Bubble
            }
            UiPointerEventKind::Move
                if input.captured_pointer_id.is_some()
                    && input.captured_pointer_id == pointer.metadata.pointer_id =>
            {
                UiInputRoutePolicy::PointerCapture
            }
            UiPointerEventKind::Move => UiInputRoutePolicy::Direct,
        },
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_) => {
            UiInputRoutePolicy::FocusPath
        }
        UiInputEvent::Navigation(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::Analog(_) => UiInputRoutePolicy::FocusPath,
        UiInputEvent::DragDrop(_) => UiInputRoutePolicy::PointerCapture,
        UiInputEvent::Popup(_) | UiInputEvent::TooltipTimer(_) | UiInputEvent::Accessibility(_) => {
            UiInputRoutePolicy::DefaultAction
        }
    }
}

pub(super) fn annotate_route_policy(
    input: &UiSurfaceInputState,
    event: &UiInputEvent,
    result: &mut zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
) {
    result.diagnostics.route_policy = route_policy_for_input_event(input, event);
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
