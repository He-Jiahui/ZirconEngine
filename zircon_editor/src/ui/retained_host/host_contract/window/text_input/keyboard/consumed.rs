use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

pub(super) fn native_keyboard_event_consumed(
    text_focus_was_active: bool,
    event: &KeyEvent,
    result: &NativePointerDispatchResult,
) -> bool {
    if result.request_redraw() {
        return true;
    }
    text_focus_was_active && text_focus_consumes_keyboard_event(event)
}

fn text_focus_consumes_keyboard_event(event: &KeyEvent) -> bool {
    if event.state != ElementState::Pressed {
        return false;
    }
    match &event.logical_key {
        Key::Named(NamedKey::Backspace | NamedKey::Escape | NamedKey::Enter) => true,
        _ => event
            .text
            .as_deref()
            .is_some_and(|text| text.chars().any(|ch| !ch.is_control())),
    }
}
