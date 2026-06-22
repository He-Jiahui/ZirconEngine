mod consumed;
mod popup;
mod unhandled;

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use consumed::native_keyboard_event_consumed;
use popup::dispatch_popup_keyboard_fallback;
use unhandled::dispatch_unhandled_keyboard_input;

impl UiHostWindow {
    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_key_event(
        &self,
        event: &KeyEvent,
    ) -> NativePointerDispatchResult {
        if event.state != ElementState::Pressed {
            return NativePointerDispatchResult::idle();
        }
        if !self.text_input_focus_active() {
            let result = dispatch_popup_keyboard_fallback(self, event);
            if result.request_redraw() {
                return result;
            }
        }
        match &event.logical_key {
            Key::Named(NamedKey::Backspace) => self.dispatch_focused_text_backspace(),
            Key::Named(NamedKey::Escape) => {
                self.global::<UiHostContext>().clear_text_input_focus();
                NativePointerDispatchResult::idle()
            }
            Key::Named(NamedKey::Enter) => self.dispatch_focused_text_commit(),
            _ => event
                .text
                .as_deref()
                .map_or_else(NativePointerDispatchResult::idle, |text| {
                    self.dispatch_focused_text_insert(text)
                }),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_keyboard_event(
        &self,
        event: &KeyEvent,
        keyboard: Option<UiKeyboardInputEvent>,
    ) -> NativePointerDispatchResult {
        let text_focus_was_active = self.text_input_focus_active();
        let result = self.dispatch_focused_key_event(event);
        if !native_keyboard_event_consumed(text_focus_was_active, event, &result) {
            dispatch_unhandled_keyboard_input(self, event, keyboard);
        }
        result
    }
}
