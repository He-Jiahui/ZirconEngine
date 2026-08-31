use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_message::DocumentId;
use crate::core::play::WorldDomain;

use super::{editor_state::EditorState, EditorStateOperationError};

impl EditorState {
    /// Receives the document identity only after the lifecycle authority has committed a scene.
    pub(crate) fn bind_scene_document(&mut self, document: DocumentId) {
        self.active_scene_document = Some(document);
    }

    /// Removes the ephemeral state binding before a world is replaced or cleared.
    pub(crate) fn clear_scene_document_binding(&mut self) {
        self.active_scene_document = None;
    }

    pub(crate) fn active_scene_history_context(&self) -> Option<HistoryContextId> {
        if self.is_playing() {
            return match self.viewport_controller.selection().active_domain() {
                WorldDomain::Play(instance) => Some(HistoryContextId::PlaySession(instance)),
                WorldDomain::Edit => None,
            };
        }
        self.active_scene_document.map(HistoryContextId::Document)
    }

    pub(crate) fn scene_history_context(
        &self,
    ) -> Result<HistoryContextId, EditorStateOperationError> {
        self.active_scene_history_context().ok_or_else(|| {
            if self.is_playing() {
                EditorStateOperationError::PlayWorldNotActive
            } else {
                EditorStateOperationError::SceneDocumentNotActive
            }
        })
    }
}
