mod consumed;
mod popup;
mod unhandled;

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use super::super::UiHostWindow;
use crate::core::commands::EditorKeyChord;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use consumed::native_keyboard_event_consumed;
use popup::{dispatch_popup_keyboard_fallback, popup_enter_is_owned};
use unhandled::dispatch_unhandled_keyboard_input;

impl UiHostWindow {
    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_key_event(
        &self,
        event: &KeyEvent,
        popup_enter_is_owned: bool,
    ) -> NativePointerDispatchResult {
        if event.state != ElementState::Pressed {
            return NativePointerDispatchResult::idle();
        }
        if !self.text_input_focus_active() {
            let result = dispatch_popup_keyboard_fallback(self, event);
            if result.request_redraw() || popup_enter_is_owned {
                return result;
            }
        }
        match &event.logical_key {
            Key::Named(NamedKey::Backspace) => self.dispatch_focused_text_backspace(),
            Key::Named(NamedKey::Escape) => self.cancel_focused_text_input(),
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
        if self.chord_capture_focus_active() {
            return self.dispatch_focused_chord_key_event(event, keyboard.as_ref());
        }
        let text_focus_was_active = self.text_input_focus_active();
        let popup_enter_is_owned = !text_focus_was_active && popup_enter_is_owned(self, event);
        let modifiers = keyboard.as_ref().map(|input| &input.metadata.modifiers);
        let control = modifiers.is_some_and(|modifiers| modifiers.control);
        let shift = modifiers.is_some_and(|modifiers| modifiers.shift);
        let result = if text_focus_was_active && event.state == ElementState::Pressed {
            match &event.logical_key {
                Key::Character(key) if control && key.eq_ignore_ascii_case("a") => {
                    self.dispatch_focused_text_caret("all", false)
                }
                Key::Named(NamedKey::ArrowLeft) => self.dispatch_focused_text_caret("left", shift),
                Key::Named(NamedKey::ArrowRight) => {
                    self.dispatch_focused_text_caret("right", shift)
                }
                Key::Named(NamedKey::Home) => self.dispatch_focused_text_caret("home", shift),
                Key::Named(NamedKey::End) => self.dispatch_focused_text_caret("end", shift),
                Key::Named(NamedKey::Delete) => self.dispatch_focused_text_delete(),
                // Preserve command routing for undo/redo and other editor shortcuts.
                Key::Character(_) if control || modifiers.is_some_and(|m| m.super_key || m.alt) => {
                    NativePointerDispatchResult::idle()
                }
                _ => self.dispatch_focused_key_event(event, popup_enter_is_owned),
            }
        } else {
            self.dispatch_focused_key_event(event, popup_enter_is_owned)
        };
        if !popup_enter_is_owned
            && !(text_focus_was_active
                && event.state == ElementState::Pressed
                && control
                && matches!(&event.logical_key, Key::Character(key) if key.eq_ignore_ascii_case("a")))
            && !native_keyboard_event_consumed(
                text_focus_was_active
                    && !(matches!(&event.logical_key, Key::Character(_))
                        && modifiers.is_some_and(|m| m.control || m.super_key || m.alt)),
                event,
                &result,
            )
        {
            dispatch_unhandled_keyboard_input(self, event, keyboard);
        }
        result
    }

    fn dispatch_focused_chord_key_event(
        &self,
        event: &KeyEvent,
        keyboard: Option<&UiKeyboardInputEvent>,
    ) -> NativePointerDispatchResult {
        if event.state != ElementState::Pressed {
            return NativePointerDispatchResult::idle();
        }
        if matches!(&event.logical_key, Key::Named(NamedKey::Escape)) {
            return self.cancel_focused_chord_capture();
        }
        let Some(chord) = keyboard.and_then(EditorKeyChord::from_keyboard_input) else {
            return NativePointerDispatchResult::idle();
        };
        self.dispatch_focused_chord_commit(chord.to_string().into())
    }
}
