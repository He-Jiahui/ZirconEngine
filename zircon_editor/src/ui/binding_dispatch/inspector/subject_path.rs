use zircon_runtime::scene::NodeId;

use super::super::error::EditorBindingDispatchError;
use crate::ui::workbench::state::EditorState;

pub(super) fn resolve_subject_path(
    state: &EditorState,
    subject_path: &str,
) -> Result<NodeId, EditorBindingDispatchError> {
    if subject_path == "entity://selected" {
        return state.viewport_controller.selected_node().ok_or_else(|| {
            EditorBindingDispatchError::InvalidSubjectPath(subject_path.to_string())
        });
    }

    if let Some(raw) = subject_path.strip_prefix("node://") {
        let node_id = raw.parse::<NodeId>().map_err(|_| {
            EditorBindingDispatchError::InvalidSubjectPath(subject_path.to_string())
        })?;
        let exists = state
            .world
            .with_world(|scene| scene.find_node(node_id).is_some());
        if exists {
            return Ok(node_id);
        }
    }

    Err(EditorBindingDispatchError::InvalidSubjectPath(
        subject_path.to_string(),
    ))
}
