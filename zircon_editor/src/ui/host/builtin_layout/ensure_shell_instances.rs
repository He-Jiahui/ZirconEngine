use crate::ui::workbench::view::{ViewInstance, ViewRegistry};

use super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::editor_error::EditorError;
use super::super::editor_session_state::EditorSessionState;
use super::builtin_shell_view_instances::builtin_shell_view_instances;

pub(crate) fn ensure_builtin_shell_instances(
    registry: &mut ViewRegistry,
    session: &mut EditorSessionState,
    snapshot: &EditorCapabilitySnapshot,
) -> Result<(), EditorError> {
    registry.set_available_capabilities(snapshot.enabled_capabilities().to_vec());
    for instance in builtin_shell_view_instances(snapshot) {
        if preserved_single_instance(registry, session, &instance) {
            continue;
        }
        let restored = restore_or_reuse_instance(registry, &instance)?;
        session
            .open_view_instances
            .insert(restored.instance_id.clone(), restored);
    }
    Ok(())
}

fn preserved_single_instance(
    registry: &ViewRegistry,
    session: &EditorSessionState,
    instance: &ViewInstance,
) -> bool {
    registry
        .descriptor(&instance.descriptor_id)
        .is_some_and(|descriptor| !descriptor.multi_instance)
        && session
            .open_view_instances
            .values()
            .any(|current| current.descriptor_id == instance.descriptor_id)
}

fn restore_or_reuse_instance(
    registry: &mut ViewRegistry,
    instance: &ViewInstance,
) -> Result<ViewInstance, EditorError> {
    if let Some(existing) = registry.instance(&instance.instance_id).cloned() {
        Ok(existing)
    } else {
        registry
            .restore_instance(instance.clone())
            .map_err(EditorError::Registry)
    }
}
