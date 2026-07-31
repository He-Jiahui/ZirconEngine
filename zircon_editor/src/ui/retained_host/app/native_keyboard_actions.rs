use super::*;
use crate::core::editor_event::EditorEventSource;
use crate::ui::retained_host::event_bridge::apply_record_effects;
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

impl RetainedEditorHost {
    pub(super) fn dispatch_unhandled_native_keyboard_input(
        &mut self,
        keyboard: UiKeyboardInputEvent,
    ) {
        if self.try_begin_hierarchy_rename_from_keyboard(&keyboard) {
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
}
