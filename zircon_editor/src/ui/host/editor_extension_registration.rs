use crate::core::editor_event::EditorEventRuntime;
use crate::core::editor_extension::{
    EditorExtensionRegistration, EditorExtensionRegistry, EditorExtensionRegistryError,
};
use crate::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, EditorOperationRegistryError,
};
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
        let views = extension.views().into_iter().cloned().collect::<Vec<_>>();
        let mut operation_registry = inner.operation_registry.clone();
        for operation in extension.operations().descriptors().cloned() {
            operation_registry
                .register(
                    operation.with_required_capabilities(required_capabilities.iter().cloned()),
                )
                .map_err(EditorExtensionRegistryError::Operation)?;
        }
        for view in &views {
            let operation_path = view
                .open_operation_path()
                .map_err(EditorExtensionRegistryError::Operation)?;
            if operation_registry.descriptor(&operation_path).is_none() {
                operation_registry
                    .register(extension_view_open_operation(
                        view,
                        operation_path,
                        &required_capabilities,
                    ))
                    .map_err(EditorExtensionRegistryError::Operation)?;
            }
        }
        let available_operations = operation_registry
            .descriptors()
            .map(|descriptor| descriptor.path().clone())
            .collect::<std::collections::BTreeSet<_>>();
        validate_menu_item_operation_bindings(&extension, &available_operations)?;
        validate_component_drawer_operation_bindings(&extension, &available_operations)?;
        validate_extension_contribution_conflicts(&inner.editor_extensions, &extension)?;
        inner
            .manager
            .validate_extension_views(&views)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        inner
            .manager
            .register_extension_views_with_required_capabilities(&views, &required_capabilities)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        inner.operation_registry = operation_registry;
        inner.editor_extensions.push(
            EditorExtensionRegistration::new(extension)
                .with_required_capabilities(required_capabilities),
        );
        Ok(())
    }
}

fn extension_view_open_operation(
    view: &crate::core::editor_extension::ViewDescriptor,
    operation_path: EditorOperationPath,
    required_capabilities: &[String],
) -> EditorOperationDescriptor {
    EditorOperationDescriptor::new(operation_path, format!("Open {}", view.display_name()))
        .with_menu_path(format!("View/{}/{}", view.category(), view.display_name()))
        .with_required_capabilities(required_capabilities.iter().cloned())
        .with_event(crate::core::editor_event::EditorEvent::WorkbenchMenu(
            crate::core::editor_event::MenuAction::OpenView(
                crate::core::editor_event::ViewDescriptorId::new(view.id()),
            ),
        ))
}

fn validate_menu_item_operation_bindings(
    extension: &EditorExtensionRegistry,
    available_operations: &std::collections::BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for menu_item in extension.menu_items() {
        if !available_operations.contains(menu_item.operation()) {
            return Err(EditorExtensionRegistryError::Operation(
                EditorOperationRegistryError::MissingOperation(menu_item.operation().clone()),
            ));
        }
    }
    Ok(())
}

fn validate_extension_contribution_conflicts(
    registrations: &[EditorExtensionRegistration],
    extension: &EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    validate_contribution_ids(
        registrations.iter().flat_map(|registration| {
            registration
                .registry()
                .drawers()
                .into_iter()
                .map(|drawer| drawer.id().to_string())
        }),
        extension
            .drawers()
            .into_iter()
            .map(|drawer| drawer.id().to_string()),
        "drawer",
    )?;
    validate_contribution_ids(
        registrations.iter().flat_map(|registration| {
            registration
                .registry()
                .menu_items()
                .into_iter()
                .map(|menu_item| menu_item.path().to_string())
        }),
        extension
            .menu_items()
            .into_iter()
            .map(|menu_item| menu_item.path().to_string()),
        "menu item",
    )?;
    validate_contribution_ids(
        registrations.iter().flat_map(|registration| {
            registration
                .registry()
                .component_drawers()
                .into_iter()
                .map(|drawer| drawer.component_type().to_string())
        }),
        extension
            .component_drawers()
            .into_iter()
            .map(|drawer| drawer.component_type().to_string()),
        "component drawer",
    )?;
    validate_contribution_ids(
        registrations.iter().flat_map(|registration| {
            registration
                .registry()
                .ui_templates()
                .into_iter()
                .map(|template| template.id().to_string())
        }),
        extension
            .ui_templates()
            .into_iter()
            .map(|template| template.id().to_string()),
        "ui template",
    )
}

fn validate_contribution_ids<I, J>(
    existing_ids: I,
    candidate_ids: J,
    kind: &'static str,
) -> Result<(), EditorExtensionRegistryError>
where
    I: IntoIterator<Item = String>,
    J: IntoIterator<Item = String>,
{
    let mut ids = existing_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for id in candidate_ids {
        if !ids.insert(id.clone()) {
            return Err(EditorExtensionRegistryError::DuplicateContribution { kind, id });
        }
    }
    Ok(())
}

fn validate_component_drawer_operation_bindings(
    extension: &EditorExtensionRegistry,
    available_operations: &std::collections::BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for component_drawer in extension.component_drawers() {
        for binding in component_drawer.bindings() {
            let path = EditorOperationPath::parse(binding.clone())
                .map_err(EditorExtensionRegistryError::Operation)?;
            if !available_operations.contains(&path) {
                return Err(EditorExtensionRegistryError::Operation(
                    EditorOperationRegistryError::MissingOperation(path),
                ));
            }
        }
    }
    Ok(())
}
