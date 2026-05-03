use super::builtin_views::builtin_view_descriptors::builtin_view_descriptors;
use super::editor_error::EditorError;
use super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(super) fn register_builtin_views(&self) -> Result<(), EditorError> {
        let mut registry = self.lock_view_registry();
        let snapshot = self.lock_capability_snapshot().clone();
        registry.set_available_capabilities(snapshot.enabled_capabilities().to_vec());
        for descriptor in builtin_view_descriptors(&snapshot) {
            if registry.descriptor(&descriptor.descriptor_id).is_none() {
                registry
                    .register_view(descriptor)
                    .map_err(EditorError::Registry)?;
            }
        }
        Ok(())
    }
}
