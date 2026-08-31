use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use crate::core::editing::engine::HistoryStatus;
use crate::core::editor_event::EditorAnimationEvent;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn apply_animation_event(&self, event: &EditorAnimationEvent) -> Result<bool, EditorError> {
        self.host.apply_animation_event(event)
    }

    pub(crate) fn replay_focused_animation_document_history(
        &self,
        undo: bool,
    ) -> Result<Option<bool>, EditorError> {
        self.host.replay_focused_animation_document_history(undo)
    }

    pub(crate) fn focused_animation_history_status(&self) -> Option<HistoryStatus> {
        self.host.focused_animation_history_status()
    }

    pub fn save_animation_editor(&self, instance_id: &ViewInstanceId) -> Result<(), EditorError> {
        self.host.save_animation_editor(instance_id)
    }

    pub fn animation_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<AnimationEditorPanePresentation, EditorError> {
        self.host.animation_editor_pane_presentation(instance_id)
    }
}
