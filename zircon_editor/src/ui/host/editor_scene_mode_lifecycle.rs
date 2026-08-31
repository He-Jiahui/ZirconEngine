use crate::core::editor_message::SceneModeId;
use crate::ui::workbench::state::EditorViewportStateError;

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn push_scene_mode_overlay(
        &self,
        mode_id: SceneModeId,
    ) -> Result<(), EditorViewportStateError> {
        let mut shell = self.shell().lock();
        shell
            .state
            .prepare_non_gizmo_scene_action()
            .map_err(EditorViewportStateError::StateMutation)?;
        shell
            .state
            .viewport_controller
            .push_scene_mode_overlay(&mode_id)
            .map_err(EditorViewportStateError::ViewportController)
    }

    pub(crate) fn pop_scene_mode_overlay(
        &self,
    ) -> Result<Option<SceneModeId>, EditorViewportStateError> {
        let mut shell = self.shell().lock();
        shell
            .state
            .prepare_non_gizmo_scene_action()
            .map_err(EditorViewportStateError::StateMutation)?;
        Ok(shell.state.viewport_controller.pop_scene_mode_overlay())
    }

    pub(crate) fn update_scene_modes(&self) {
        self.shell()
            .lock()
            .state
            .viewport_controller
            .update_scene_modes();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::editor_message::SceneModeId;
    use crate::ui::workbench::state::EditorViewportStateError;

    use super::EditorHostEventController;

    #[test]
    fn scene_mode_lifecycle_keeps_viewport_state_error_contracts_typed() {
        let _: fn(&EditorHostEventController, SceneModeId) -> Result<(), EditorViewportStateError> =
            EditorHostEventController::push_scene_mode_overlay;
        let _: fn(
            &EditorHostEventController,
        ) -> Result<Option<SceneModeId>, EditorViewportStateError> =
            EditorHostEventController::pop_scene_mode_overlay;
    }
}
