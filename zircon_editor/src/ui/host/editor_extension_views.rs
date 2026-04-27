use crate::core::editor_extension::ViewDescriptor as ExtensionViewDescriptor;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use super::editor_ui_host::EditorUiHost;

impl EditorManager {
    pub fn register_extension_view(
        &self,
        descriptor: &ExtensionViewDescriptor,
    ) -> Result<(), EditorError> {
        self.host
            .register_extension_view_with_required_capabilities(descriptor, &[])
    }

    pub fn register_extension_view_with_required_capabilities(
        &self,
        descriptor: &ExtensionViewDescriptor,
        required_capabilities: &[String],
    ) -> Result<(), EditorError> {
        self.host
            .register_extension_view_with_required_capabilities(descriptor, required_capabilities)
    }
}

impl EditorUiHost {
    pub(super) fn register_extension_view_with_required_capabilities(
        &self,
        descriptor: &ExtensionViewDescriptor,
        required_capabilities: &[String],
    ) -> Result<(), EditorError> {
        let mut view = ViewDescriptor::new(
            ViewDescriptorId::new(descriptor.id()),
            ViewKind::ActivityView,
            descriptor.display_name(),
        )
        .with_icon_key(descriptor.id());
        view.required_capabilities = required_capabilities.to_vec();

        let mut registry = self.view_registry.lock().unwrap();
        if registry.descriptor(&view.descriptor_id).is_some() {
            return Ok(());
        }
        registry.register_view(view).map_err(EditorError::Registry)
    }
}
