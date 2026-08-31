use zircon_runtime_interface::ui::{
    dispatch::{UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent},
    surface::UiHitTestQuery,
    tree::UiTreeError,
};

use super::super::surface::UiSurface;
use super::{
    accessibility::dispatch_accessibility_input,
    analog::dispatch_analog_input,
    diagnostics_budget::{diagnostics_budget_required, enforce_diagnostics_budget},
    drag_drop::dispatch_drag_drop_input,
    editable_text::{dispatch_ime_input, dispatch_text_input},
    keyboard::dispatch_keyboard_input,
    keyboard_clipboard::dispatch_clipboard_input,
    mouse_motion::dispatch_mouse_motion_input,
    navigation::dispatch_navigation_input,
    pointer::dispatch_pointer_input,
    popup::dispatch_popup_input,
    route_authority::annotate_authoritative_input_dispatch,
    submenu_hover_timer::dispatch_submenu_hover_timer_input,
    toast_timer::dispatch_toast_timer_input,
    tooltip_timer::dispatch_tooltip_timer_input,
    typeahead_timer::dispatch_typeahead_timer_input,
};
use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher, UiTextDocumentSession};

pub(crate) fn dispatch_input_event(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    navigation_dispatcher: &UiNavigationDispatcher,
    event: UiInputEvent,
    pointer_query: Option<UiHitTestQuery>,
    mut text_documents: Option<&mut UiTextDocumentSession>,
    diagnostics_mode: UiInputDiagnosticsMode,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let mut result = match event {
        UiInputEvent::Pointer(pointer) => dispatch_pointer_input(
            surface,
            pointer_dispatcher,
            pointer,
            pointer_query,
            diagnostics_mode,
        )?,
        UiInputEvent::Navigation(navigation) => {
            dispatch_navigation_input(surface, navigation_dispatcher, navigation, diagnostics_mode)?
        }
        UiInputEvent::Keyboard(keyboard) => dispatch_keyboard_input(
            surface,
            navigation_dispatcher,
            keyboard,
            text_documents.as_deref_mut(),
            move |surface, dispatcher, navigation| {
                dispatch_navigation_input(surface, dispatcher, navigation, diagnostics_mode)
            },
        )?,
        UiInputEvent::Text(text) => {
            dispatch_text_input(surface, text, text_documents.as_deref_mut())
        }
        UiInputEvent::Ime(ime) => dispatch_ime_input(surface, ime, text_documents.as_deref_mut()),
        UiInputEvent::Clipboard(clipboard) => {
            dispatch_clipboard_input(surface, clipboard, text_documents.as_deref_mut())
        }
        UiInputEvent::Analog(analog) => dispatch_analog_input(
            surface,
            navigation_dispatcher,
            analog,
            diagnostics_mode,
            move |surface, dispatcher, navigation| {
                dispatch_navigation_input(surface, dispatcher, navigation, diagnostics_mode)
            },
        )?,
        UiInputEvent::MouseMotion(motion) => {
            dispatch_mouse_motion_input(surface, motion, diagnostics_mode)
        }
        UiInputEvent::DragDrop(drag_drop) => {
            dispatch_drag_drop_input(surface, drag_drop, diagnostics_mode)
        }
        UiInputEvent::Popup(popup) => dispatch_popup_input(surface, popup),
        UiInputEvent::TooltipTimer(tooltip) => dispatch_tooltip_timer_input(surface, tooltip),
        UiInputEvent::TypeaheadTimer(typeahead) => {
            dispatch_typeahead_timer_input(surface, typeahead)
        }
        UiInputEvent::SubmenuHoverTimer(submenu_hover) => {
            dispatch_submenu_hover_timer_input(surface, submenu_hover)
        }
        UiInputEvent::ToastTimer(toast) => dispatch_toast_timer_input(surface, toast),
        UiInputEvent::Accessibility(accessibility) => {
            dispatch_accessibility_input(surface, accessibility, text_documents.as_deref_mut())
        }
    };
    surface
        .input
        .append_deferred_focus_input_lifecycle(&mut result, usize::MAX);
    surface.redact_secure_text_dispatch_result(&mut result);
    if diagnostics_mode.captures_full_trace() {
        annotate_authoritative_input_dispatch(&mut result);
        enforce_diagnostics_budget(&mut result);
    } else if diagnostics_budget_required(&result) {
        enforce_diagnostics_budget(&mut result);
    }
    Ok(result)
}
