use super::builtin_views::builtin_view_descriptors;
use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub(super) fn register_builtin_views(&self) -> Result<(), EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        for descriptor in builtin_view_descriptors() {
            if registry.descriptor(&descriptor.descriptor_id).is_none() {
                registry
                    .register_view(descriptor)
                    .map_err(EditorError::Registry)?;
            }
        }
        Ok(())
    }
}
