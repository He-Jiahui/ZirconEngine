use std::collections::HashSet;

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
    let mut open_descriptor_ids = session
        .open_view_instances
        .values()
        .map(|instance| instance.descriptor_id.clone())
        .collect::<HashSet<_>>();
    for instance in builtin_shell_view_instances(snapshot) {
        if preserved_single_instance(registry, &open_descriptor_ids, &instance) {
            continue;
        }
        let restored = restore_or_reuse_instance(registry, &instance)?;
        let restored_descriptor_id = restored.descriptor_id.clone();
        let replaced = session
            .open_view_instances
            .insert(restored.instance_id.clone(), restored);
        open_descriptor_ids.insert(restored_descriptor_id.clone());
        if let Some(replaced) = replaced {
            let replaced_descriptor_id = replaced.descriptor_id;
            if replaced_descriptor_id != restored_descriptor_id
                && !session
                    .open_view_instances
                    .values()
                    .any(|current| current.descriptor_id == replaced_descriptor_id)
            {
                open_descriptor_ids.remove(&replaced_descriptor_id);
            }
        }
    }
    Ok(())
}

fn preserved_single_instance(
    registry: &ViewRegistry,
    open_descriptor_ids: &HashSet<crate::ui::workbench::view::ViewDescriptorId>,
    instance: &ViewInstance,
) -> bool {
    registry
        .descriptor(&instance.descriptor_id)
        .is_some_and(|descriptor| !descriptor.multi_instance)
        && open_descriptor_ids.contains(&instance.descriptor_id)
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

#[cfg(test)]
#[path = "ensure_shell_instances/indexed_preservation_tests.rs"]
mod indexed_preservation_tests;
