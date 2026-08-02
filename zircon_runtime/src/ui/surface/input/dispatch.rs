use zircon_runtime_interface::ui::{
    dispatch::{UiInputDispatchResult, UiInputEvent},
    tree::UiTreeError,
};

use super::super::surface::UiSurface;
use super::{
    accessibility::dispatch_accessibility_input,
    analog::dispatch_analog_input,
    drag_drop::dispatch_drag_drop_input,
    editable_text::{dispatch_ime_input, dispatch_text_input},
    keyboard::dispatch_keyboard_input,
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
use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};

pub(crate) fn dispatch_input_event(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    navigation_dispatcher: &UiNavigationDispatcher,
    event: UiInputEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let mut result = match event {
        UiInputEvent::Pointer(pointer) => {
            dispatch_pointer_input(surface, pointer_dispatcher, pointer)?
        }
        UiInputEvent::Navigation(navigation) => {
            dispatch_navigation_input(surface, navigation_dispatcher, navigation)?
        }
        UiInputEvent::Keyboard(keyboard) => dispatch_keyboard_input(
            surface,
            navigation_dispatcher,
            keyboard,
            dispatch_navigation_input,
        )?,
        UiInputEvent::Text(text) => dispatch_text_input(surface, text),
        UiInputEvent::Ime(ime) => dispatch_ime_input(surface, ime),
        UiInputEvent::Analog(analog) => dispatch_analog_input(
            surface,
            navigation_dispatcher,
            analog,
            dispatch_navigation_input,
        )?,
        UiInputEvent::MouseMotion(motion) => dispatch_mouse_motion_input(surface, motion),
        UiInputEvent::DragDrop(drag_drop) => dispatch_drag_drop_input(surface, drag_drop),
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
            dispatch_accessibility_input(surface, accessibility)
        }
    };
    surface
        .input
        .append_deferred_focus_input_lifecycle(&mut result, usize::MAX);
    annotate_authoritative_input_dispatch(&mut result);
    Ok(result)
}
