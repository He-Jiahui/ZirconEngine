use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::ui::workbench::view::ViewInstanceId;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    /// Replays only a focused animation document. A non-animation focus leaves the caller to
    /// route the request to the scene's global history.
    pub(super) fn replay_focused_animation_document_history(
        &self,
        undo: bool,
    ) -> Result<Option<bool>, EditorError> {
        let Some(instance_id) = self.focused_animation_editor_instance() else {
            return Ok(None);
        };
        self.ensure_animation_editor_session(&instance_id)?;
        let document = self.animation_document_for_instance(&instance_id)?;
        let changed = if undo {
            self.transactions.undo(HistoryContextId::Document(document))
        } else {
            self.transactions.redo(HistoryContextId::Document(document))
        }
        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        if changed {
            self.reconcile_animation_session_after_source_replay(&instance_id);
            self.sync_animation_editor_instance(&instance_id)?;
        }
        Ok(Some(changed))
    }

    /// Returns the active animation history for command enablement without materializing a
    /// document from disk during a snapshot request.
    pub(super) fn focused_animation_history_status(&self) -> Option<HistoryStatus> {
        let instance_id = self.focused_animation_editor_instance()?;
        let document = self
            .lock_animation_editor_sessions()
            .get(&instance_id)
            .map(|entry| entry.document)?;
        self.transactions
            .history_status(HistoryContextId::Document(document))
            .ok()
    }

    fn focused_animation_editor_instance(&self) -> Option<ViewInstanceId> {
        let session = self.lock_session();
        let instance_id = session.focused_view.clone()?;
        session
            .open_view_instances
            .get(&instance_id)
            .filter(|instance| {
                matches!(
                    instance.descriptor_id.0.as_str(),
                    "editor.animation_sequence" | "editor.animation_graph"
                )
            })
            .map(|_| instance_id)
    }

    fn reconcile_animation_session_after_source_replay(&self, instance_id: &ViewInstanceId) {
        let mut sessions = self.lock_animation_editor_sessions();
        if let Some(entry) = sessions.get_mut(instance_id) {
            entry.session.reconcile_source_change();
        }
    }
}
