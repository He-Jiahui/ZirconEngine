use crate::core::editor_message::SceneModeId;

use super::EditorHostEventController;

impl EditorHostEventController {
    pub fn push_scene_mode_overlay(&self, mode_id: SceneModeId) -> Result<(), String> {
        let mut shell = self.shell().lock();
        shell.state.prepare_non_gizmo_scene_action()?;
        shell
            .state
            .viewport_controller
            .push_scene_mode_overlay(&mode_id)
    }

    pub fn pop_scene_mode_overlay(&self) -> Result<Option<SceneModeId>, String> {
        let mut shell = self.shell().lock();
        shell.state.prepare_non_gizmo_scene_action()?;
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
