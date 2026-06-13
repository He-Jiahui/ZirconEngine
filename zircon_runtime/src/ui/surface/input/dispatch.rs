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
    submenu_hover_timer::dispatch_submenu_hover_timer_input,
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
    match event {
        UiInputEvent::Pointer(pointer) => {
            dispatch_pointer_input(surface, pointer_dispatcher, pointer)
        }
        UiInputEvent::Navigation(navigation) => {
            let result = dispatch_navigation_input(surface, navigation_dispatcher, navigation)?;
            Ok(result)
        }
        UiInputEvent::Keyboard(keyboard) => dispatch_keyboard_input(
            surface,
            navigation_dispatcher,
            keyboard,
            dispatch_navigation_input,
        ),
        UiInputEvent::Text(text) => Ok(dispatch_text_input(surface, text)),
        UiInputEvent::Ime(ime) => Ok(dispatch_ime_input(surface, ime)),
        UiInputEvent::Analog(analog) => dispatch_analog_input(
            surface,
            navigation_dispatcher,
            analog,
            dispatch_navigation_input,
        ),
        UiInputEvent::MouseMotion(motion) => Ok(dispatch_mouse_motion_input(surface, motion)),
        UiInputEvent::DragDrop(drag_drop) => Ok(dispatch_drag_drop_input(surface, drag_drop)),
        UiInputEvent::Popup(popup) => Ok(dispatch_popup_input(surface, popup)),
        UiInputEvent::TooltipTimer(tooltip) => Ok(dispatch_tooltip_timer_input(surface, tooltip)),
        UiInputEvent::TypeaheadTimer(typeahead) => {
            Ok(dispatch_typeahead_timer_input(surface, typeahead))
        }
        UiInputEvent::SubmenuHoverTimer(submenu_hover) => {
            Ok(dispatch_submenu_hover_timer_input(surface, submenu_hover))
        }
        UiInputEvent::Accessibility(accessibility) => {
            Ok(dispatch_accessibility_input(surface, accessibility))
        }
    }
}
