use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::asset::AssetTypeRegistry;
use crate::core::commands::{
    project_command_registry_from_contributions, NativePluginEditorCommandBinding,
};
use crate::core::editor_extension::{
    EditorContributionHandle, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
    EditorUiTemplatePaneDataSource,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{
    CapabilitySet, ContributionBatch, ContributionError, ContributionSource, PluginContributionId,
};
use crate::core::plugin::run_editor_plugin_boundary;
use crate::core::plugin::EditorPluginRegistrationReport;
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;
use crate::core::tools::{ToolDefinitionId, ToolInstanceId, ToolOwnerGeneration};
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::{OwnedContribution, WorkbenchShellStateData};

impl EditorHostEventController {
    pub fn register_editor_extension(
        &self,
        extension: ContributionBatch,
    ) -> Result<(), EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.register_editor_extension_owned(
            "editor.extension.direct",
            ContributionSource::Builtin,
            extension,
            Vec::<String>::new(),
            None,
            BTreeMap::new(),
        )
        .map(|_| ())
    }

    pub fn register_editor_plugin_registration(
        &self,
        registration: EditorPluginRegistrationReport,
    ) -> Result<EditorContributionHandle, EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owner_id = registration.package_manifest.id;
        let source = ContributionSource::Plugin(
            PluginContributionId::parse(owner_id.clone()).map_err(map_contribution_error)?,
        );
        let extension = registration.extensions.into_contribution_batch()?;
        let native_command_bindings = registration.native_command_bindings;
        self.register_editor_extension_owned(
            owner_id,
            source,
            extension,
            registration.capabilities,
            Some(registration.runtime_event_consumers),
            native_command_bindings,
        )
    }

    /// Serializes a native live-host action with contribution publication and rejects the action
    /// while the package still owns an exact editor contribution generation. The native backend
    /// cannot replace that generation atomically yet, so bypassing this gate would split loader
    /// and Store/router authority across different dynamic-library generations.
    pub(crate) fn execute_native_live_action_without_active_contribution<T>(
        &self,
        owner_id: &str,
        execute: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let shell = self.shell().lock();
        ensure_native_live_action_has_no_active_contribution(&shell.contribution_owners, owner_id)?;
        drop(shell);
        execute()
    }

    /// Allocates a tool instance for one exact live contribution generation.
    pub fn allocate_editor_tool_instance(
        &self,
        handle: &EditorContributionHandle,
        definition_id: &ToolDefinitionId,
    ) -> Result<ToolInstanceId, EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owner_generation = self
            .shell()
            .lock()
            .contribution_owners
            .iter()
            .find(|contribution| contribution.handle() == handle)
            .map(|contribution| contribution.owner_generation())
            .ok_or_else(|| EditorExtensionRegistryError::StaleContributionHandle {
                owner_id: handle.owner_id().to_owned(),
            })?;
        self.context()
            .tools()
            .allocate_instance_id(definition_id, owner_generation)
            .map_err(|error| EditorExtensionRegistryError::ToolScheduler(error.to_string()))
    }

    /// Atomically replaces one registered plugin's template and pane-data contributions.
    pub fn replace_editor_plugin_ui_template_contributions(
        &self,
        owner_id: &str,
        templates: impl IntoIterator<Item = EditorUiTemplateDescriptor>,
        pane_data_sources: BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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

    /// Revokes one plugin's Store ticket and rebuilds executable command routing from the
    /// remaining active contribution generations.
    pub fn revoke_editor_plugin_contribution(
        &self,
        handle: &EditorContributionHandle,
    ) -> Result<bool, EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut shell = self.shell().lock();
        let matching = shell
            .contribution_owners
            .iter()
            .enumerate()
            .filter(|(_, contribution)| contribution.handle() == handle)
            .map(|(index, contribution)| {
                (
                    index,
                    contribution.ticket(),
                    contribution.owner_generation(),
                )
            })
            .collect::<Vec<_>>();
        let (owner_index, ticket, owner_generation) = match matching.as_slice() {
            [] => return Ok(false),
            [owned] => *owned,
            _ => {
                return Err(EditorExtensionRegistryError::DuplicateContribution {
                    kind: "editor extension owner",
                    id: handle.owner_id().to_string(),
                });
            }
        };

        let view_descriptor_ids = shell
            .contributions
            .snapshot()
            .views_for_ticket(ticket)
            .map(|view| crate::ui::workbench::view::ViewDescriptorId::new(view.id()))
            .collect::<Vec<_>>();
        let manager = Arc::clone(&shell.manager);

        let mut contribution_store = shell.contributions.clone();
        contribution_store.revoke(ticket);
        let previous_command_generation = self.commands().lock().generation();
        let command_registry = project_command_registry_from_contributions(
            &contribution_store,
            previous_command_generation,
        )?;
        let prepared_scene_mode_retirement = shell
            .state
            .viewport_controller
            .prepare_scene_mode_contribution_retirement(ticket);
        drop(shell);
        manager
            .retire_extension_views(&view_descriptor_ids)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        let runtime_consumer_retirement = self
            .runtime_event_consumers
            .retire_contribution(ticket)
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        if owner_generation != ToolOwnerGeneration::BUILTIN {
            self.context()
                .tools()
                .revoke_owner_generation(owner_generation)
                .map_err(|error| {
                    EditorExtensionRegistryError::ToolScheduler(format!(
                        "tool owner generation {owner_generation} retirement failed: {error}"
                    ))
                })?;
        }

        let mut shell = self.shell().lock();
        let (retired_scene_modes, scene_mode_cleanup_error) = shell
            .state
            .viewport_controller
            .install_prepared_scene_mode_contribution_retirement(prepared_scene_mode_retirement);
        let (prepared_viewport_overlay_providers, viewport_overlay_provider_retirement) = shell
            .state
            .viewport_controller
            .prepare_viewport_overlay_provider_retirement(ticket);
        shell
            .state
            .viewport_controller
            .install_prepared_viewport_overlay_providers(prepared_viewport_overlay_providers);
        shell.contributions = contribution_store;
        shell.contribution_owners.remove(owner_index);
        shell.contributions_changed();
        *self.commands().lock() = command_registry;
        drop(shell);
        drop(retired_scene_modes);
        let viewport_overlay_provider_cleanup = viewport_overlay_provider_retirement.cleanup();
        self.refresh_workbench(
            crate::core::editor_message::EditorViewInvalidationMask::PRESENTATION_DATA,
        );
        if let Some(error) = runtime_consumer_retirement.cleanup_error {
            return Err(EditorExtensionRegistryError::View(format!(
                "runtime event consumer retirement failed: {error}"
            )));
        }
        if let Some(error) = scene_mode_cleanup_error {
            return Err(EditorExtensionRegistryError::SceneMode(format!(
                "scene mode retirement failed: {error}"
            )));
        }
        viewport_overlay_provider_cleanup.map_err(|error| {
            EditorExtensionRegistryError::ViewportOverlayProvider(format!(
                "viewport overlay provider retirement failed: {error}"
            ))
        })?;
        Ok(true)
    }

    pub fn register_editor_extension_with_required_capabilities(
        &self,
        extension: ContributionBatch,
        required_capabilities: Vec<String>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.register_editor_extension_owned(
            "editor.extension.direct",
            ContributionSource::Builtin,
            extension,
            required_capabilities,
            None,
            BTreeMap::new(),
        )
        .map(|_| ())
    }

    fn register_editor_extension_owned(
        &self,
        owner_id: impl Into<String>,
        source: ContributionSource,
        mut extension: ContributionBatch,
        required_capabilities: Vec<String>,
        runtime_event_consumers: Option<EditorRuntimeEventConsumerRegistry>,
        native_command_bindings: BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>,
    ) -> Result<EditorContributionHandle, EditorExtensionRegistryError> {
        let owner_id = owner_id.into();
        if let Some((command_id, binding)) = native_command_bindings
            .iter()
            .find(|(_, binding)| binding.plugin_id() != owner_id.as_str())
        {
            return Err(validate_native_binding_owner(
                command_id,
                &owner_id,
                binding.plugin_id(),
            ));
        }
        extension = extension
            .with_required_capabilities(required_capabilities.clone())
            .with_native_command_bindings(native_command_bindings);
        extension.bind_matching_ui_templates_to_views();
        let mut shell = self.shell().lock();
        if matches!(&source, ContributionSource::Plugin(_))
            && shell
                .contribution_owners
                .iter()
                .any(|contribution| contribution.owner_id() == owner_id)
        {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "editor extension owner",
                id: owner_id,
            });
        }
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
        let mut contribution_store = shell.contributions.clone();
        let ticket = contribution_store
            .contribute(source.clone(), contributed_extension)
            .map_err(map_contribution_error)?;
        let scene_modes = extension
            .take_scene_modes()
            .into_iter()
            .map(|registration| {
                registration.bind_contribution_owner(ticket, source.clone(), owner_id.clone())
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EditorExtensionRegistryError::SceneMode(error.to_string()))?;
        let prepared_scene_modes = shell
            .state
            .viewport_controller
            .prepare_scene_modes(scene_modes)
            .map_err(|error| EditorExtensionRegistryError::SceneMode(error.to_string()))?;
        let viewport_overlay_providers = extension.take_viewport_overlay_providers();
        let prepared_viewport_overlay_providers =
            run_editor_plugin_boundary(&owner_id, "viewport overlay provider preparation", || {
                shell
                    .state
                    .viewport_controller
                    .prepare_viewport_overlay_providers(
                        ticket,
                        source.clone(),
                        &owner_id,
                        viewport_overlay_providers,
                    )
                    .map_err(|error| error.to_string())
            })
            .map_err(|error| {
                EditorExtensionRegistryError::ViewportOverlayProvider(error.to_string())
            })?;
        let prepared_runtime_event_consumers = runtime_event_consumers
            .map(|registry| {
                self.runtime_event_consumers
                    .prepare_contribution_registration(ticket, source.clone(), registry)
            })
            .transpose()
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        let previous_command_generation = self.commands().lock().generation();
        let command_registry = project_command_registry_from_contributions(
            &contribution_store,
            previous_command_generation,
        )?;
        let tool_resource_kinds = extension.tool_resource_kinds().cloned().collect::<Vec<_>>();
        let owner_generation = match &source {
            ContributionSource::Builtin => ToolOwnerGeneration::BUILTIN,
            ContributionSource::Plugin(_) => *self
                .context()
                .tools()
                .register_owner_generation(tool_resource_kinds)
                .map_err(|error| {
                    EditorExtensionRegistryError::ToolScheduler(format!(
                        "tool owner generation registration failed: {error}"
                    ))
                })?
                .outcome(),
        };
        if let Err(error) = shell
            .manager
            .register_extension_views_with_required_capabilities(&views, &required_capabilities)
        {
            drop(shell);
            if owner_generation != ToolOwnerGeneration::BUILTIN {
                let _ = self
                    .context()
                    .tools()
                    .revoke_owner_generation(owner_generation);
            }
            return Err(EditorExtensionRegistryError::View(error.to_string()));
        }
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
        shell.contributions = contribution_store;
        let contribution_handle = EditorContributionHandle::new(owner_id, ticket, owner_generation);
        shell
            .contribution_owners
            .push(crate::ui::workbench::shell_state::OwnedContribution::new(
                contribution_handle.clone(),
            ));
        shell.contributions_changed();
        *self.commands().lock() = command_registry;
        if let Some(registry) = prepared_runtime_event_consumers {
            self.runtime_event_consumers
                .install_prepared_registration(registry);
        }
        drop(shell);
        Ok(contribution_handle)
    }
}

fn validate_native_binding_owner(
    command_id: &EditorOperationPath,
    expected_plugin_id: &str,
    binding_plugin_id: &str,
) -> EditorExtensionRegistryError {
    EditorExtensionRegistryError::View(format!(
        "native editor command `{command_id}` binds plugin `{binding_plugin_id}`, expected contribution owner `{expected_plugin_id}`"
    ))
}

fn ensure_native_live_action_has_no_active_contribution(
    owners: &[OwnedContribution],
    owner_id: &str,
) -> Result<(), String> {
    if owners
        .iter()
        .any(|contribution| contribution.owner_id() == owner_id)
    {
        return Err(format!(
            "native editor plugin `{owner_id}` has an active exact contribution generation; unload and hot reload require a generation-aware contribution transaction"
        ));
    }
    Ok(())
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
        ContributionError::PluginNamespace {
            plugin_id,
            kind: "tool resource kind",
            id,
        } => EditorExtensionRegistryError::ToolResourceKindOwnerMismatch {
            expected_prefix: format!("plugin.{}.", plugin_id.as_str()),
            owner_id: plugin_id.as_str().to_owned(),
            kind: id,
        },
        ContributionError::ToolResourceKindRequiresPluginSource { kind } => {
            EditorExtensionRegistryError::ToolResourceKindRequiresPluginSource { kind }
        }
        error => EditorExtensionRegistryError::View(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        EditorCommandExecutionContract, EditorCommandResourceBudget, EditorCommandResultCodecId,
    };

    use super::{
        ensure_native_live_action_has_no_active_contribution,
        project_command_registry_from_contributions,
    };
    use crate::core::commands::{
        EditorCommandDescriptor, EditorCommandExecutorRegistryError, EditorCommandRegistryError,
    };
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::{ContributionBatch, ContributionSource, ContributionStore};
    use crate::core::tools::ToolOwnerGeneration;
    use crate::ui::workbench::shell_state::OwnedContribution;

    #[test]
    fn native_live_action_requires_a_generation_aware_contribution_transaction() {
        let mut contributions = ContributionStore::default();
        let ticket = contributions
            .contribute(ContributionSource::Builtin, ContributionBatch::default())
            .unwrap();
        let handle = crate::core::editor_extension::EditorContributionHandle::new(
            "fixture.native-editor",
            ticket,
            ToolOwnerGeneration::BUILTIN,
        );
        let owners = [OwnedContribution::new(handle)];

        let error =
            ensure_native_live_action_has_no_active_contribution(&owners, "fixture.native-editor")
                .expect_err("live action must not bypass an active exact contribution generation");

        assert!(error.contains("fixture.native-editor"));
        assert!(error.contains("generation-aware contribution transaction"));
        assert!(ensure_native_live_action_has_no_active_contribution(
            &owners,
            "fixture.runtime-only"
        )
        .is_ok());
    }

    #[test]
    fn native_endpoint_without_same_batch_binding_is_rejected_before_projection() {
        let command_id = EditorOperationPath::parse("fixture.editor.native").unwrap();
        let descriptor = EditorCommandDescriptor::native(command_id).with_execution_contract(
            EditorCommandExecutionContract::new(
                EditorCommandResultCodecId::parse("zircon.editor.result.v1").unwrap(),
                EditorCommandResourceBudget::new(1024, 1024, 1000).unwrap(),
            ),
        );
        let mut batch = ContributionBatch::default();
        batch.register_command(descriptor).unwrap();
        let mut contributions = ContributionStore::default();
        contributions
            .contribute(ContributionSource::Builtin, batch)
            .unwrap();

        let error = project_command_registry_from_contributions(&contributions, 0)
            .expect_err("native endpoint without an admitted binding must not publish");
        assert!(matches!(
            error,
            crate::core::editor_extension::EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::Executor(
                    EditorCommandExecutorRegistryError::MissingExecutor { .. }
                )
            )
        ));
    }

    #[test]
    fn native_binding_owner_must_match_contribution_owner() {
        let command_id = EditorOperationPath::parse("fixture.editor.native").unwrap();
        let error =
            super::validate_native_binding_owner(&command_id, "fixture.editor", "other.editor");
        assert!(matches!(
            error,
            crate::core::editor_extension::EditorExtensionRegistryError::View(message)
                if message.contains("other.editor")
                    && message.contains("fixture.editor")
        ));
    }
}
