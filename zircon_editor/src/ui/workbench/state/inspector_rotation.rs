use zircon_runtime::scene::{NodeId, Scene};

use crate::core::editing::{command::EditorCommand, euler_rotation::with_local_xyz_degrees};
use crate::ui::workbench::state::{EditorStateOperationError, InspectorEditError};

pub(super) fn capture_rotation_command(
    scene: &Scene,
    node_id: NodeId,
    axes: [Option<f32>; 3],
) -> Result<Option<EditorCommand>, EditorStateOperationError> {
    if axes.iter().all(Option::is_none) {
        return Ok(None);
    }
    let mut transform = scene
        .local_transform(node_id)
        .ok_or(EditorStateOperationError::SelectedNodeMissing { node_id })?;
    transform.rotation = with_local_xyz_degrees(transform.rotation, axes)
        .ok_or(InspectorEditError::InvalidRotationQuaternion)?;
    // Keep raw reflected quaternion access read-only. The existing transform command owns
    // domain validation, invalidation, transaction rollback, journal replay and undo/redo.
    Ok(EditorCommand::set_transform(scene, node_id, transform)?)
}
