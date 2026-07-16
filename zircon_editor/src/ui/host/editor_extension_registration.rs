use crate::core::asset::{AssetContextCommandAccess, AssetTypeRegistry};
use crate::core::commands::{
    AssetWriteTargetDescriptor, EditorCommandDescriptor, EditorCommandMenuProjection,
    EditorCommandRegistryError,
};
use crate::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use crate::core::editor_extension::{
    EditorExtensionRegistration, EditorExtensionRegistry, EditorExtensionRegistryError,
    ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::editor_plugin::EditorPluginRegistrationReport;
use crate::ui::host::EditorHostEventController;

impl EditorHostEventController {
    pub fn register_editor_extension(
        &self,
        extension: EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_editor_extension_owned(
            "editor.extension.direct",
            extension,
            Vec::<String>::new(),
        )
    }

    pub fn register_editor_plugin_registration(
        &self,
        registration: EditorPluginRegistrationReport,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_runtime_event_consumers(registration.runtime_event_consumers)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        self.register_editor_extension_owned(
            registration.package_manifest.id,
            registration.extensions,
            registration.capabilities,
        )
    }

    pub fn register_editor_extension_with_required_capabilities(
        &self,
        extension: EditorExtensionRegistry,
        required_capabilities: Vec<String>,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_editor_extension_owned(
            "editor.extension.direct",
            extension,
            required_capabilities,
        )
    }

    fn register_editor_extension_owned(
        &self,
        owner_id: impl Into<String>,
        mut extension: EditorExtensionRegistry,
        required_capabilities: Vec<String>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let owner_id = owner_id.into();
        let mut shell = self.shell().lock();
        let views = extension.views().into_iter().cloned().collect::<Vec<_>>();
        shell
            .manager
            .validate_extension_views(&views)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        validate_asset_type_contributions(&shell.editor_extensions, &extension, &owner_id)?;
        validate_extension_contribution_conflicts(&shell.editor_extensions, &extension)?;
        let mut command_registry = self.commands().lock().clone();
        let menu_capabilities = extension
            .menu_items()
            .into_iter()
            .filter(|item| !item.required_capabilities().is_empty())
            .fold(
                std::collections::BTreeMap::<EditorOperationPath, Vec<String>>::new(),
                |mut capabilities, item| {
                    capabilities
                        .entry(item.operation().clone())
                        .or_default()
                        .extend(item.required_capabilities().iter().cloned());
                    capabilities
                },
            );
        let pending_command_ids = extension
            .pending_commands()
            .map(|command| command.id().clone())
            .collect::<std::collections::BTreeSet<_>>();
        let asset_write_targets = asset_write_targets(&extension)?;
        let view_operation_ids = views
            .iter()
            .map(ViewDescriptor::open_operation_path)
            .collect::<Result<std::collections::BTreeSet<_>, _>>()
            .map_err(EditorExtensionRegistryError::OperationPath)?;
        if let Some(command_id) = menu_capabilities.keys().find(|command_id| {
            !pending_command_ids.contains(*command_id) && !view_operation_ids.contains(*command_id)
        }) {
            return Err(
                EditorExtensionRegistryError::MenuCapabilitiesRequireContributedCommand {
                    command_id: command_id.clone(),
                },
            );
        }
        let commands = extension.take_command_contributions();
        let mut operation_factories = extension
            .take_operation_factories()
            .into_iter()
            .map(|factory| (factory.operation().clone(), factory))
            .collect::<std::collections::BTreeMap<_, _>>();
        let explicit_view_commands = commands
            .iter()
            .map(|command| (command.id().clone(), command.event().cloned()))
            .collect::<std::collections::BTreeMap<_, _>>();
        for command in commands {
            let command_capabilities = menu_capabilities
                .get(command.id())
                .into_iter()
                .flatten()
                .cloned();
            let command = command
                .with_required_capabilities(required_capabilities.iter().cloned())
                .with_required_capabilities(command_capabilities);
            if let Some(factory) = operation_factories.remove(command.id()) {
                command_registry
                    .register_operation(command, factory)
                    .map_err(EditorExtensionRegistryError::Command)?;
            } else {
                command_registry
                    .register(command)
                    .map_err(EditorExtensionRegistryError::Command)?;
            }
        }
        if let Some(operation) = operation_factories.keys().next().cloned() {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::OperationFactory(
                    crate::core::editing::operation::OperationCommandFactoryError::OrphanFactory {
                        operation,
                    },
                ),
            ));
        }
        for view in &views {
            let operation_path = view
                .open_operation_path()
                .map_err(EditorExtensionRegistryError::OperationPath)?;
            let expected_event = extension_view_open_event(view);
            if let Some(explicit_event) = explicit_view_commands.get(&operation_path) {
                if explicit_event.as_ref() != Some(&expected_event) {
                    return Err(EditorExtensionRegistryError::CommandViewTargetConflict {
                        command_id: operation_path,
                        view_id: view.id().to_string(),
                    });
                }
            } else if command_registry.command(operation_path.as_str()).is_some() {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::DuplicateCommand(operation_path),
                ));
            } else {
                let mut view_capabilities = required_capabilities.clone();
                view_capabilities.extend(
                    menu_capabilities
                        .get(&operation_path)
                        .into_iter()
                        .flatten()
                        .cloned(),
                );
                command_registry
                    .register(extension_view_open_operation(
                        view,
                        operation_path.clone(),
                        &view_capabilities,
                    ))
                    .map_err(EditorExtensionRegistryError::Command)?;
            }
            extension.record_registered_command_id(operation_path);
        }
        for (operation, target) in asset_write_targets {
            command_registry
                .attach_asset_write_target(&operation, target)
                .map_err(EditorExtensionRegistryError::Command)?;
        }
        let available_operations = command_registry
            .commands()
            .map(|descriptor| descriptor.id().clone())
            .collect::<std::collections::BTreeSet<_>>();
        validate_menu_item_operation_bindings(&extension, &available_operations)?;
        validate_component_drawer_operation_bindings(&extension, &available_operations)?;
        validate_asset_importer_operation_bindings(&extension, &available_operations)?;
        validate_asset_type_operation_bindings(&extension, &available_operations)?;
        shell
            .manager
            .register_extension_views_with_required_capabilities(&views, &required_capabilities)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        shell.editor_extensions.push(
            EditorExtensionRegistration::new(extension)
                .with_owner_id(owner_id)
                .with_required_capabilities(required_capabilities),
        );
        *self.commands().lock() = command_registry;
        drop(shell);
        Ok(())
    }
}

fn asset_write_targets(
    extension: &EditorExtensionRegistry,
) -> Result<
    std::collections::BTreeMap<EditorOperationPath, AssetWriteTargetDescriptor>,
    EditorExtensionRegistryError,
> {
    let mut targets = std::collections::BTreeMap::new();
    for contribution in extension.asset_type_contributions() {
        for template in contribution.creation_templates() {
            insert_asset_write_target(
                &mut targets,
                template.operation().clone(),
                AssetWriteTargetDescriptor::new("asset_type", "target_folder"),
            )?;
        }
        for command in contribution
            .context_commands()
            .iter()
            .filter(|command| command.access() == AssetContextCommandAccess::Mutation)
        {
            insert_asset_write_target(
                &mut targets,
                command.operation().clone(),
                AssetWriteTargetDescriptor::new("asset_type", "asset_locator"),
            )?;
        }
    }
    Ok(targets)
}

fn insert_asset_write_target(
    targets: &mut std::collections::BTreeMap<EditorOperationPath, AssetWriteTargetDescriptor>,
    operation: EditorOperationPath,
    target: AssetWriteTargetDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    if targets
        .get(&operation)
        .is_some_and(|existing| existing != &target)
    {
        return Err(EditorExtensionRegistryError::Command(
            EditorCommandRegistryError::ConflictingAssetWriteTarget(operation),
        ));
    }
    targets.insert(operation, target);
    Ok(())
}

fn extension_view_open_operation(
    view: &ViewDescriptor,
    operation_path: EditorOperationPath,
    required_capabilities: &[String],
) -> EditorCommandDescriptor {
    EditorCommandDescriptor::operation(operation_path, format!("Open {}", view.display_name()))
        .with_menu_path(format!("View/{}/{}", view.category(), view.display_name()))
        .with_menu_projection(EditorCommandMenuProjection::ExtensionRegistry)
        .with_required_capabilities(required_capabilities.iter().cloned())
        .with_event(extension_view_open_event(view))
}

fn extension_view_open_event(view: &ViewDescriptor) -> EditorEvent {
    EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view.id())))
}

fn validate_menu_item_operation_bindings(
    extension: &EditorExtensionRegistry,
    available_operations: &std::collections::BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for menu_item in extension.menu_items() {
        if !available_operations.contains(menu_item.operation()) {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::MissingCommand(menu_item.operation().clone()),
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
    )?;
    validate_contribution_ids(
        registrations.iter().flat_map(|registration| {
            registration
                .registry()
                .asset_importers()
                .into_iter()
                .map(|importer| importer.id().to_string())
        }),
        extension
            .asset_importers()
            .into_iter()
            .map(|importer| importer.id().to_string()),
        "asset importer",
    )?;
    Ok(())
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
                .map_err(EditorExtensionRegistryError::OperationPath)?;
            if !available_operations.contains(&path) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(path),
                ));
            }
        }
    }
    Ok(())
}

fn validate_asset_importer_operation_bindings(
    extension: &EditorExtensionRegistry,
    available_operations: &std::collections::BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for importer in extension.asset_importers() {
        if !available_operations.contains(importer.operation()) {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::MissingCommand(importer.operation().clone()),
            ));
        }
    }
    Ok(())
}

fn validate_asset_type_operation_bindings(
    extension: &EditorExtensionRegistry,
    available_operations: &std::collections::BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for contribution in extension.asset_type_contributions() {
        if let Some(toolkit) = contribution.toolkit() {
            if !available_operations.contains(toolkit.open_operation()) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(toolkit.open_operation().clone()),
                ));
            }
        }
        for template in contribution.creation_templates() {
            if !available_operations.contains(template.operation()) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(template.operation().clone()),
                ));
            }
        }
        for command in contribution.context_commands() {
            if !available_operations.contains(command.operation()) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(command.operation().clone()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_asset_type_contributions(
    registrations: &[EditorExtensionRegistration],
    candidate: &EditorExtensionRegistry,
    candidate_owner: &str,
) -> Result<(), EditorExtensionRegistryError> {
    let mut asset_types = AssetTypeRegistry::with_builtins()?;
    for registration in registrations {
        for contribution in registration.registry().asset_type_contributions() {
            asset_types.apply_contribution(registration.owner_id(), (*contribution).clone())?;
        }
    }
    for contribution in candidate.asset_type_contributions() {
        asset_types.apply_contribution(candidate_owner, (*contribution).clone())?;
    }
    Ok(())
}

pub(crate) fn materialize_enabled_asset_types(
    registrations: &[EditorExtensionRegistration],
    enabled_capabilities: &[String],
) -> Result<AssetTypeRegistry, EditorExtensionRegistryError> {
    let mut asset_types = AssetTypeRegistry::with_builtins()?;
    for registration in registrations
        .iter()
        .filter(|registration| registration.is_enabled_by(enabled_capabilities))
    {
        for contribution in registration.registry().asset_type_contributions() {
            asset_types.apply_contribution(registration.owner_id(), (*contribution).clone())?;
        }
    }
    Ok(asset_types)
}
