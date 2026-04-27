use crate::core::editor_event::EditorEventRuntime;
use crate::core::editor_extension::{
    EditorExtensionRegistration, EditorExtensionRegistry, EditorExtensionRegistryError,
};
use crate::core::editor_operation::EditorOperationDescriptor;
use crate::core::editor_plugin::EditorPluginRegistrationReport;

impl EditorEventRuntime {
    pub fn register_editor_extension(
        &self,
        extension: EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_editor_extension_with_required_capabilities(extension, Vec::<String>::new())
    }

    pub fn register_editor_plugin_registration(
        &self,
        registration: EditorPluginRegistrationReport,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_editor_extension_with_required_capabilities(
            registration.extensions,
            registration.capabilities,
        )
    }

    pub fn register_editor_extension_with_required_capabilities(
        &self,
        extension: EditorExtensionRegistry,
        required_capabilities: Vec<String>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let mut inner = self.inner.lock().unwrap();
        for operation in extension.operations().descriptors().cloned() {
            inner
                .operation_registry
                .register(
                    operation.with_required_capabilities(required_capabilities.iter().cloned()),
                )
                .map_err(EditorExtensionRegistryError::Operation)?;
        }
        for view in extension.views() {
            let operation_path = view
                .open_operation_path()
                .map_err(EditorExtensionRegistryError::Operation)?;
            if inner
                .operation_registry
                .descriptor(&operation_path)
                .is_none()
            {
                inner
                    .operation_registry
                    .register(
                        EditorOperationDescriptor::new(
                            operation_path,
                            format!("Open {}", view.display_name()),
                        )
                        .with_menu_path(format!("View/{}/{}", view.category(), view.display_name()))
                        .with_required_capabilities(required_capabilities.iter().cloned())
                        .with_event(
                            crate::core::editor_event::EditorEvent::WorkbenchMenu(
                                crate::core::editor_event::MenuAction::OpenView(
                                    crate::core::editor_event::ViewDescriptorId::new(view.id()),
                                ),
                            ),
                        ),
                    )
                    .map_err(EditorExtensionRegistryError::Operation)?;
            }
            inner
                .manager
                .register_extension_view_with_required_capabilities(view, &required_capabilities)
                .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        }
        inner.editor_extensions.push(
            EditorExtensionRegistration::new(extension)
                .with_required_capabilities(required_capabilities),
        );
        Ok(())
    }
}
