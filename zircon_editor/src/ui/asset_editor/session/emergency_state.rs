use crate::ui::asset_editor::{UiAssetEditorCommand, UiAssetEditorShellState};

use super::ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError};

impl UiAssetEditorSession {
    pub fn shell_state(&self) -> UiAssetEditorShellState {
        if self.diagnostics.is_empty() {
            UiAssetEditorShellState::Valid
        } else if self.preview_host.is_some() {
            UiAssetEditorShellState::Emergency
        } else {
            UiAssetEditorShellState::Invalid
        }
    }

    pub fn emergency_summary(&self) -> String {
        self.diagnostics.first().cloned().unwrap_or_default()
    }

    pub fn can_revert_to_last_valid_source(&self) -> bool {
        self.source_buffer.text() != self.last_valid_source_text
    }

    pub fn revert_source_to_last_valid(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        if !self.can_revert_to_last_valid_source() {
            return Ok(false);
        }
        let source = self.last_valid_source_text.clone();
        self.apply_command(UiAssetEditorCommand::edit_source(source))?;
        Ok(true)
    }
}
