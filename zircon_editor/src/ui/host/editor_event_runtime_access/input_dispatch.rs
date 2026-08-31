use crate::core::editor_event::{EditorEventRecord, EditorEventSource};
use crate::ui::host::EditorHostEventController;
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchDisposition, UiInputDispatchResult, UiInputEvent, UiKeyboardInputEvent,
};

impl EditorHostEventController {
    pub(crate) fn dispatch_unhandled_input_keymap_command(
        &self,
        result: &UiInputDispatchResult,
        source: EditorEventSource,
    ) -> Result<Option<EditorEventRecord>, String> {
        if result.reply.disposition != UiDispatchDisposition::Unhandled {
            return Ok(None);
        }
        let UiInputEvent::Keyboard(keyboard) = &result.event else {
            return Ok(None);
        };
        self.dispatch_keyboard_keymap_command(keyboard, source)
    }

    pub(crate) fn dispatch_keyboard_keymap_command(
        &self,
        keyboard: &UiKeyboardInputEvent,
        source: EditorEventSource,
    ) -> Result<Option<EditorEventRecord>, String> {
        let Some(command_id) = self.shell().lock().manager.resolve_keyboard_input(keyboard) else {
            return Ok(None);
        };
        self.dispatch_keymap_command_id(&command_id, source)
            .map(Some)
    }

    fn dispatch_keymap_command_id(
        &self,
        command_id: &str,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        let binding = crate::ui::binding::EditorUiBinding::new(
            "EditorKeymap",
            command_id,
            crate::ui::binding::EditorUiEventKind::Submit,
            crate::ui::binding::EditorUiBindingPayload::editor_command(command_id),
        );
        self.dispatch_binding(binding, source)
            .map_err(|error| error.to_string())
    }
}
