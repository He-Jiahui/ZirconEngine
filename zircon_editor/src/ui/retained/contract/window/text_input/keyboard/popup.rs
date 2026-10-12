use winit::event::ElementState;
use winit::event::KeyEvent;

use super::super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::native_keyboard::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
    workbench_popup_accept_is_owned, workbench_popup_keyboard_command,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

pub(super) fn popup_enter_is_owned(window: &UiHostWindow, event: &KeyEvent) -> bool {
    event.state == ElementState::Pressed
        && workbench_popup_accept_is_owned(window, &event.logical_key)
}

pub(super) fn dispatch_popup_keyboard_fallback(
    window: &UiHostWindow,
    event: &KeyEvent,
) -> NativePointerDispatchResult {
    if let Some(command) = workbench_popup_keyboard_command(&event.logical_key) {
        let result = dispatch_workbench_popup_keyboard_command(window, command);
        if result.request_redraw() {
            return result;
        }
    }
    if let Some(text) = event.text.as_deref() {
        let result = dispatch_workbench_popup_text_search(window, text);
        if result.request_redraw() {
            return result;
        }
    }
    NativePointerDispatchResult::idle()
}
