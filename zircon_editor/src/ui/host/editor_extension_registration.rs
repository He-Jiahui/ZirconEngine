use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::asset::{AssetContextCommandAccess, AssetTypeRegistry};
use crate::core::commands::{
    AssetWriteTargetDescriptor, EditorCommandDescriptor, EditorCommandMenuProjection,
    EditorCommandRegistryError,
};
use crate::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use crate::core::editor_extension::{
    EditorExtensionRegistryError, EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSource,
    ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{
    CapabilitySet, ContributionBatch, ContributionError, ContributionSource, PluginContributionId,
};
use crate::core::plugin::run_editor_plugin_boundary;
use crate::core::plugin::EditorPluginRegistrationReport;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

impl EditorHostEventController {
    pub fn register_editor_extension(
        &self,
        extension: ContributionBatch,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_editor_extension_owned(
            "editor.extension.direct",
            ContributionSource::Builtin,
            extension,
            Vec::<String>::new(),
        )
    }

    pub fn register_editor_plugin_registration(
        &self,
        registration: EditorPluginRegistrationReport,
    ) -> Result<(), EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let prepared_consumers = self
            .runtime_event_consumers
            .prepare_registration(registration.runtime_event_consumers)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        let owner_id = registration.package_manifest.id;
        let source = ContributionSource::Plugin(
            PluginContributionId::parse(owner_id.clone()).map_err(map_contribution_error)?,
        );
        let extension = registration.extensions.into_contribution_batch()?;
        self.register_editor_extension_owned(
            owner_id,
            source,
            extension,
            registration.capabilities,
        )?;
        self.runtime_event_consumers
            .install_prepared_registration(prepared_consumers);
        Ok(())
    }

    /// Atomically replaces one registered plugin's template and pane-data contributions.
    pub fn replace_editor_plugin_ui_template_contributions(
        &self,
        owner_id: &str,
        templates: impl IntoIterator<Item = EditorUiTemplateDescriptor>,
        pane_data_sources: BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let mut shell = self.shell().lock();
        let mut matching = shell
            .contribution_owners
            .iter()
            .filter(|contribution| contribution.owner_id() == owner_id)
            .map(|contribution| contribution.ticket());
        let ticket = match (matching.next(), matching.next()) {
            (None, _) => {
                return Err(EditorExtensionRegistryError::UnknownExtensionOwner {
                    owner_id: owner_id.to_string(),
                });
            }
            (Some(_), Some(_)) => {
                return Err(EditorExtensionRegistryError::DuplicateContribution {
                    kind: "editor extension owner",
                    id: owner_id.to_string(),
                });
            }
            (Some(ticket), None) => ticket,
        };
        shell
            .contributions
            .replace_ui_template_contributions(ticket, templates, pane_data_sources)
            .map_err(map_contribution_error)?;
        drop(shell);
        self.refresh_workbench(
            crate::core::editor_message::EditorViewInvalidationMask::PRESENTATION_DATA,
        );
        Ok(())
    }

    pub fn register_editor_extension_with_required_capabilities(
        &self,
        extension: ContributionBatch,
        required_capabilities: Vec<String>,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.register_editor_extension_owned(
            "editor.extension.direct",
            ContributionSource::Builtin,
            extension,
            required_capabilities,
        )
    }

    fn register_editor_extension_owned(
        &self,
        owner_id: impl Into<String>,
        source: ContributionSource,
        mut extension: ContributionBatch,
        required_capabilities: Vec<String>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let owner_id = owner_id.into();
        extension = extension.with_required_capabilities(required_capabilities.clone());
        extension.bind_matching_ui_templates_to_views();
        let mut shell = self.shell().lock();
        let views = extension.views().into_iter().cloned().collect::<Vec<_>>();
        shell
            .manager
            .validate_extension_views(&views)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        validate_asset_type_contributions(&shell.contributions.snapshot(), &extension, &owner_id)?;
        shell
            .contributions
            .validate_contribution(&source, &extension)
            .map_err(map_contribution_error)?;
        validate_viewport_overlay_provider_bindings(&extension)?;
        let contributed_extension = extension.clone();
        let scene_modes = extension
            .take_scene_modes()
            .into_iter()
            .map(|registration| registration.with_owner_id(owner_id.clone()))
            .collect::<Vec<_>>();
        let prepared_scene_modes = shell
            .state
            .viewport_controller
            .prepare_scene_modes(scene_modes)
            .map_err(EditorExtensionRegistryError::SceneMode)?;
        let viewport_overlay_providers = extension.take_viewport_overlay_providers();
        let prepared_viewport_overlay_providers =
            run_editor_plugin_boundary(&owner_id, "viewport overlay provider preparation", || {
                shell
                    .state
                    .viewport_controller
                    .prepare_viewport_overlay_providers(&owner_id, viewport_overlay_providers)
            })
            .map_err(|error| {
                EditorExtensionRegistryError::ViewportOverlayProvider(error.to_string())
            })?;
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
        validate_inspector_customization_operation_bindings(&extension, &available_operations)?;
        validate_asset_importer_operation_bindings(&extension, &available_operations)?;
        validate_asset_type_operation_bindings(&extension, &available_operations)?;
        shell
            .manager
            .register_extension_views_with_required_capabilities(&views, &required_capabilities)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        shell
            .state
            .viewport_controller
            .install_prepared_viewport_overlay_providers(prepared_viewport_overlay_providers);
        shell
            .state
            .viewport_controller
            .install_prepared_scene_modes(prepared_scene_modes);
        let enabled_capabilities = shell
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        shell
            .state
            .viewport_controller
            .set_viewport_overlay_capabilities(&enabled_capabilities);
        let ticket = shell
            .contributions
            .contribute(source, contributed_extension)
            .map_err(map_contribution_error)?;
        shell
            .contribution_owners
            .push(crate::ui::workbench::shell_state::OwnedContribution::new(
                owner_id, ticket,
            ));
        shell.contributions_changed();
        *self.commands().lock() = command_registry;
        drop(shell);
        Ok(())
    }
}

fn validate_viewport_overlay_provider_bindings(
    extension: &ContributionBatch,
) -> Result<(), EditorExtensionRegistryError> {
    for descriptor in extension.scene_mode_descriptors() {
        let Some(provider_id) = descriptor.overlay_provider_id() else {
            continue;
        };
        if !extension.has_viewport_overlay_provider(provider_id) {
            return Err(
                EditorExtensionRegistryError::MissingViewportOverlayProvider {
                    provider_id: provider_id.to_string(),
                },
            );
        }
    }
    Ok(())
}

fn asset_write_targets(
    extension: &ContributionBatch,
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
    extension: &ContributionBatch,
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

fn validate_inspector_customization_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &std::collections::BTreeSet<EditorOperationPath>,
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
    extension: &ContributionBatch,
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
    snapshot: &crate::core::extension::ContributionSnapshot,
    candidate: &ContributionBatch,
    candidate_owner: &str,
) -> Result<(), EditorExtensionRegistryError> {
    let mut asset_types = AssetTypeRegistry::with_builtins()?;
    for (source, contribution) in snapshot.all_asset_type_contributions_with_source() {
        asset_types.apply_contribution(contribution_owner(source), contribution.clone())?;
    }
    for contribution in candidate.asset_type_contributions() {
        asset_types.apply_contribution(candidate_owner, contribution.clone())?;
    }
    Ok(())
}

pub(crate) fn materialize_enabled_asset_types(
    snapshot: &crate::core::extension::ContributionSnapshot,
    enabled_capabilities: &[String],
) -> Result<AssetTypeRegistry, EditorExtensionRegistryError> {
    let mut asset_types = AssetTypeRegistry::with_builtins()?;
    let capabilities = enabled_capabilities
        .iter()
        .cloned()
        .collect::<CapabilitySet>();
    let report = asset_types.apply_contributions(
        snapshot
            .asset_type_contributions_with_source(&capabilities)
            .map(|(source, contribution)| (contribution_owner(source), contribution.clone())),
    );
    if let Some((_, error)) = report.into_errors().into_iter().next() {
        return Err(error.into());
    }
    Ok(asset_types)
}

pub(crate) fn enabled_asset_types_for_shell(
    shell: &mut WorkbenchShellStateData,
) -> Result<Arc<AssetTypeRegistry>, EditorExtensionRegistryError> {
    let enabled_capabilities = shell
        .manager
        .capability_snapshot()
        .enabled_capabilities()
        .to_vec();
    if let Some(registry) = shell.asset_type_registry_cache.get(&enabled_capabilities) {
        return Ok(registry);
    }
    let registry = Arc::new(materialize_enabled_asset_types(
        &shell.contributions.snapshot(),
        &enabled_capabilities,
    )?);
    shell
        .asset_type_registry_cache
        .store(enabled_capabilities, Arc::clone(&registry));
    Ok(registry)
}

fn contribution_owner(source: &ContributionSource) -> &str {
    match source {
        ContributionSource::Builtin => "editor.builtin",
        ContributionSource::Plugin(plugin_id) => plugin_id.as_str(),
    }
}

fn map_contribution_error(error: ContributionError) -> EditorExtensionRegistryError {
    match error {
        ContributionError::Batch(error) => error,
        ContributionError::DuplicateContribution { kind, id } => {
            EditorExtensionRegistryError::DuplicateContribution { kind, id }
        }
        error => EditorExtensionRegistryError::View(error.to_string()),
    }
}
