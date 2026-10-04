use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::asset::AssetContextCommandAccess;
use crate::core::editing::operation::{
    OperationCommandFactoryError, OperationCommandFactoryRegistration,
};
use crate::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use crate::core::editor_extension::{EditorExtensionRegistryError, ViewDescriptor};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{ContributionBatch, ContributionStore};

use super::{
    AssetWriteTargetDescriptor, EditorCommandAction, EditorCommandDescriptor,
    EditorCommandExecutorRegistryError, EditorCommandMenuPath, EditorCommandMenuProjection,
    EditorCommandRegistry, EditorCommandRegistryError,
};

/// One-shot command descriptors plus the stable ids retained after registration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandContributionSet {
    command_ids: BTreeSet<EditorOperationPath>,
    pending: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    #[serde(skip)]
    pending_factories: BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
}

impl EditorCommandContributionSet {
    pub fn register(
        &mut self,
        descriptor: EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        EditorCommandRegistry::validate_descriptor(&descriptor)?;
        let command_id = self.claim_command_id(descriptor.id())?;
        self.pending.insert(command_id, descriptor);
        Ok(())
    }

    pub fn register_operation(
        &mut self,
        descriptor: EditorCommandDescriptor,
        factory: OperationCommandFactoryRegistration,
    ) -> Result<(), EditorCommandRegistryError> {
        if descriptor.id() != factory.operation() {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::OperationMismatch {
                    descriptor_operation: descriptor.id().clone(),
                    factory_operation: factory.operation().clone(),
                },
            ));
        }
        if !matches!(descriptor.action(), EditorCommandAction::Operation) {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DescriptorIsEvent {
                    operation: descriptor.id().clone(),
                },
            ));
        }
        EditorCommandRegistry::validate_descriptor(&descriptor)?;
        let operation = self.claim_command_id(descriptor.id())?;
        if self.pending_factories.contains_key(factory.operation()) {
            self.command_ids.remove(&operation);
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DuplicateFactory {
                    operation: factory.operation().clone(),
                },
            ));
        }
        self.pending.insert(operation.clone(), descriptor);
        self.pending_factories.insert(operation, factory);
        Ok(())
    }

    pub fn command_ids(&self) -> impl Iterator<Item = &EditorOperationPath> {
        self.command_ids.iter()
    }

    pub fn pending_command(&self, id: &EditorOperationPath) -> Option<&EditorCommandDescriptor> {
        self.pending.get(id)
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.pending.values()
    }

    pub fn pending_factory(
        &self,
        id: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.pending_factories.get(id)
    }

    pub fn take_pending(&mut self) -> Vec<EditorCommandDescriptor> {
        std::mem::take(&mut self.pending).into_values().collect()
    }

    pub fn take_pending_factories(&mut self) -> Vec<OperationCommandFactoryRegistration> {
        std::mem::take(&mut self.pending_factories)
            .into_values()
            .collect()
    }

    pub(crate) fn record_registered_id(&mut self, id: EditorOperationPath) {
        self.command_ids.insert(id);
    }

    fn claim_command_id(
        &mut self,
        id: &EditorOperationPath,
    ) -> Result<EditorOperationPath, EditorCommandRegistryError> {
        let command_id = id.clone();
        if !self.command_ids.insert(command_id.clone()) {
            return Err(EditorCommandRegistryError::DuplicateCommand(command_id));
        }
        Ok(command_id)
    }
}

/// Rebuilds the executable command registry from Store-owned contribution batches.
///
/// The Store remains the only contribution lifetime authority. Callers publish the returned
/// registry only after the surrounding host transaction has prepared every other projection.
pub(crate) fn project_command_registry_from_contributions(
    contributions: &ContributionStore,
    previous_generation: u64,
) -> Result<EditorCommandRegistry, EditorExtensionRegistryError> {
    let mut command_registry = EditorCommandRegistry::default_workbench();
    for extension in contributions.active_batches() {
        project_extension_commands(&mut command_registry, extension)?;
    }
    command_registry.publish_projection_after(previous_generation);
    Ok(command_registry)
}

fn project_extension_commands(
    command_registry: &mut EditorCommandRegistry,
    source_extension: &ContributionBatch,
) -> Result<(), EditorExtensionRegistryError> {
    let required_capabilities = source_extension.required_capabilities();
    let menu_capabilities = source_extension
        .menu_items()
        .into_iter()
        .filter(|item| !item.required_capabilities().is_empty())
        .fold(
            BTreeMap::<EditorOperationPath, Vec<String>>::new(),
            |mut capabilities, item| {
                capabilities
                    .entry(item.operation().clone())
                    .or_default()
                    .extend(item.required_capabilities().iter().cloned());
                capabilities
            },
        );
    let pending_command_ids = source_extension
        .pending_commands()
        .map(|command| command.id().clone())
        .collect::<BTreeSet<_>>();
    let native_command_ids = source_extension
        .native_command_bindings()
        .map(|(command_id, _)| command_id.clone())
        .collect::<BTreeSet<_>>();
    let asset_write_targets = asset_write_targets(source_extension)?;
    let view_operation_ids = source_extension
        .views()
        .map(ViewDescriptor::open_operation_path)
        .collect::<Result<BTreeSet<_>, _>>()
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
    let commands = source_extension
        .pending_commands()
        .cloned()
        .collect::<Vec<_>>();
    let explicit_view_commands = commands
        .iter()
        .map(|command| (command.id().clone(), command.event().cloned()))
        .collect::<BTreeMap<_, _>>();
    for command in commands {
        if matches!(command.action(), EditorCommandAction::NativeEndpoint)
            && !native_command_ids.contains(command.id())
        {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::Executor(
                    EditorCommandExecutorRegistryError::MissingExecutor {
                        command_id: command.id().clone(),
                    },
                ),
            ));
        }
        let command_capabilities = menu_capabilities
            .get(command.id())
            .into_iter()
            .flatten()
            .cloned();
        let command = command
            .with_required_capabilities(required_capabilities.iter().map(String::as_str))
            .with_required_capabilities(command_capabilities);
        if let Some(factory) = source_extension.operation_factory(command.id()).cloned() {
            command_registry
                .register_operation(command, factory)
                .map_err(EditorExtensionRegistryError::Command)?;
        } else {
            command_registry
                .register(command)
                .map_err(EditorExtensionRegistryError::Command)?;
        }
    }
    for (command_id, binding) in source_extension.native_command_bindings() {
        command_registry
            .register_native_executor(command_id, binding.clone())
            .map_err(|error| {
                EditorExtensionRegistryError::Command(EditorCommandRegistryError::Executor(error))
            })?;
    }
    for view in source_extension.views() {
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
            let view_capabilities = required_capabilities.iter().map(String::as_str).chain(
                menu_capabilities
                    .get(&operation_path)
                    .into_iter()
                    .flatten()
                    .map(String::as_str),
            );
            command_registry
                .register(extension_view_open_operation(
                    view,
                    operation_path.clone(),
                    view_capabilities,
                ))
                .map_err(EditorExtensionRegistryError::Command)?;
        }
    }
    for (operation, target) in asset_write_targets {
        command_registry
            .attach_asset_write_target(&operation, target)
            .map_err(EditorExtensionRegistryError::Command)?;
    }
    let available_operations = command_registry
        .commands()
        .map(|descriptor| descriptor.id().clone())
        .collect::<BTreeSet<_>>();
    validate_menu_item_operation_bindings(source_extension, &available_operations)?;
    validate_inspector_customization_operation_bindings(source_extension, &available_operations)?;
    validate_asset_importer_operation_bindings(source_extension, &available_operations)?;
    validate_asset_type_operation_bindings(source_extension, &available_operations)
}

fn asset_write_targets(
    extension: &ContributionBatch,
) -> Result<BTreeMap<EditorOperationPath, AssetWriteTargetDescriptor>, EditorExtensionRegistryError>
{
    let mut targets = BTreeMap::new();
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
    targets: &mut BTreeMap<EditorOperationPath, AssetWriteTargetDescriptor>,
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

fn extension_view_open_operation<I, S>(
    view: &ViewDescriptor,
    operation_path: EditorOperationPath,
    required_capabilities: I,
) -> EditorCommandDescriptor
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let menu_path = EditorCommandMenuPath::builtin(&operation_path, "view", &["extensions"]);
    EditorCommandDescriptor::operation(operation_path)
        .with_menu_path(menu_path)
        .with_menu_projection(EditorCommandMenuProjection::ExtensionRegistry)
        .with_required_capabilities(required_capabilities)
        .with_event(extension_view_open_event(view))
}

fn extension_view_open_event(view: &ViewDescriptor) -> EditorEvent {
    EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view.id())))
}

fn validate_menu_item_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
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

fn validate_inspector_customization_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for customization in extension.inspector_customizations() {
        let Some(surface) = customization.surface() else {
            continue;
        };
        for binding in surface.bindings() {
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
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
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
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
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

#[cfg(test)]
#[path = "tests/contribution_optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "contribution/tests/optimization_batch_ht_editor602_tests.rs"]
mod optimization_batch_ht_editor602_tests;
