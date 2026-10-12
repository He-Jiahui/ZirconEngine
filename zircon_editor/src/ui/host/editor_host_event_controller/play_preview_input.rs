use zircon_runtime_interface::ZrRuntimeEventV1;

use crate::core::play::PlayPreviewInputError;

use super::{EditorError, EditorHostEventController};

impl EditorHostEventController {
    pub(crate) fn play_preview_view_focused(&self) -> Result<bool, EditorError> {
        let manager = self.enter_editor_manager()?;
        self.shell.lock().play_preview_view_focused(&manager)
    }

    pub(crate) fn route_play_preview_input(
        &self,
        event: ZrRuntimeEventV1,
    ) -> Result<bool, PlayPreviewInputError> {
        self.play_sessions.route_preview_input(event)
    }

    pub(crate) fn play_preview_input_active(&self) -> bool {
        self.play_sessions.preview_input_active()
    }
}
