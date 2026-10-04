use crate::core::editor_message::SceneModeId;
use crate::ui::workbench::state::EditorViewportStateError;

use super::EditorHostEventController;

#[test]
fn scene_mode_lifecycle_keeps_viewport_state_error_contracts_typed() {
    let _: fn(&EditorHostEventController, SceneModeId) -> Result<(), EditorViewportStateError> =
        EditorHostEventController::push_scene_mode_overlay;
    let _: fn(&EditorHostEventController) -> Result<Option<SceneModeId>, EditorViewportStateError> =
        EditorHostEventController::pop_scene_mode_overlay;
}
