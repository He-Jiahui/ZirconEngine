use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use crate::core::editor_event::EditorAnimationEvent;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn apply_animation_event(&self, event: &EditorAnimationEvent) -> Result<bool, EditorError> {
        self.host.apply_animation_event(event)
    }

    pub fn animation_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<AnimationEditorPanePresentation, EditorError> {
        self.host.animation_editor_pane_presentation(instance_id)
    }
}
