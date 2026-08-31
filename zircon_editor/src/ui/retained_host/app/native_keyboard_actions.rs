use super::*;
use crate::core::editor_event::{EditorEventSource, EditorViewportEvent};
use crate::ui::retained_host::event_bridge::apply_record_effects;
use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

impl RetainedEditorHost {
    pub(super) fn dispatch_unhandled_native_keyboard_input(
        &mut self,
        keyboard: UiKeyboardInputEvent,
    ) {
        if self.route_focused_game_keyboard_input(&keyboard) {
            return;
        }
        if keyboard.state != UiKeyboardInputState::Pressed {
            return;
        }
        if self.try_begin_hierarchy_rename_from_keyboard(&keyboard) {
            return;
        }
        if is_escape_pressed(&keyboard) {
            self.cancel_viewport_interaction();
            return;
        }
        match self
            .runtime
            .dispatch_keyboard_keymap_command(&keyboard, EditorEventSource::RetainedHost)
        {
            Ok(Some(record)) => {
                let mut effects = UiHostEventEffects::default();
                apply_record_effects(&mut effects, &record);
                self.apply_dispatch_effects(effects);
            }
            Ok(None) => {}
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn cancel_viewport_interaction(&mut self) {
        match callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::CancelInteraction,
        ) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }
}

fn is_escape_pressed(keyboard: &UiKeyboardInputEvent) -> bool {
    keyboard.state == UiKeyboardInputState::Pressed
        && (keyboard.logical_key.eq_ignore_ascii_case("escape")
            || keyboard.logical_key.eq_ignore_ascii_case("esc")
            || keyboard.key_code == 27)
}
