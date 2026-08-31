use std::sync::Arc;

use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::core::asset::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetTypeDefinition,
    AssetTypeId, AssetTypeRegistry,
};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editor_event::EditorEventRecord;
use crate::core::editor_extension::EditorExtensionRegistryError;
use crate::core::editor_message::{DocumentId, EditorViewInvalidationMask};
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationSource};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetDetailsGeneration,
};
use crate::ui::host::editor_extension_registration::enabled_asset_types_for_shell;
use crate::ui::host::{EditorHostEventController, EditorOperationDispatchError};
use crate::ui::workbench::snapshot::{
    AssetOperationProjectionSnapshot, AssetTypeProjectionSnapshot, AssetWorkspaceItemGeneration,
    AssetWorkspaceSnapshot, EditorDataSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::state::EditorStateOperationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorAssetOperationInvokeError {
    #[error(transparent)]
    ExtensionRegistry(#[from] EditorExtensionRegistryError),
    #[error("asset creation template `{template_id}` is not registered for `{asset_type}")]
    CreationTemplateNotRegistered {
        asset_type: AssetTypeId,
        template_id: String,
    },
    #[error("asset context command `{command_id}` is not registered for `{asset_type}")]
    ContextCommandNotRegistered {
        asset_type: AssetTypeId,
        command_id: String,
    },
    #[error(transparent)]
    Operation(#[from] EditorOperationDispatchError),
}

impl EditorHostEventController {
    /// Binds a scene document after the lifecycle authority has committed its activation.
    pub(crate) fn bind_scene_document(&self, document: DocumentId) {
        self.shell().lock().state.bind_scene_document(document);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub(crate) fn active_scene_history_context(
        &self,
    ) -> Option<crate::core::editing::engine::HistoryContextId> {
        self.shell().lock().state.active_scene_history_context()
    }

    pub fn set_session_mode(&self, session_mode: EditorSessionMode) {
        self.shell().lock().state.set_session_mode(session_mode);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn set_welcome_snapshot(&self, welcome: WelcomePaneSnapshot) {
        self.shell().lock().state.set_welcome_snapshot(welcome);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_catalog(&self, catalog: Arc<EditorAssetCatalogGeneration>) {
        self.sync_asset_catalog_data(catalog);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub(crate) fn sync_asset_catalog_data(&self, catalog: Arc<EditorAssetCatalogGeneration>) {
        self.shell().lock().state.sync_asset_catalog(catalog);
    }

    pub(crate) fn sync_asset_catalog_changes(
        &self,
        catalog: Arc<EditorAssetCatalogGeneration>,
        changed_asset_uuids: &[String],
    ) {
        self.shell()
            .lock()
            .state
            .sync_asset_catalog_changes(catalog, changed_asset_uuids);
    }

    pub fn sync_asset_resources(
        &self,
        resources: Arc<zircon_runtime::core::resource::ResourceManagementGeneration>,
    ) -> bool {
        let changed = self.sync_asset_resources_data(resources);
        if changed {
            self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
        }
        changed
    }

    pub(crate) fn sync_asset_resources_data(
        &self,
        resources: Arc<zircon_runtime::core::resource::ResourceManagementGeneration>,
    ) -> bool {
        self.shell().lock().state.sync_asset_resources(resources)
    }

    pub(crate) fn sync_asset_resource_changes(
        &self,
        resources: Arc<zircon_runtime::core::resource::ResourceManagementGeneration>,
        changed_locators: &[String],
    ) -> bool {
        self.shell()
            .lock()
            .state
            .sync_asset_resource_changes(resources, changed_locators)
    }

    pub fn sync_asset_details(&self, details: Option<Arc<EditorAssetDetailsGeneration>>) {
        self.shell().lock().state.sync_asset_details(details);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn replace_world(
        &self,
        world: impl Into<AuthoringWorldSeed>,
        project_path: impl Into<String>,
    ) -> Result<(), EditorStateOperationError> {
        self.shell()
            .lock()
            .state
            .replace_world(world, project_path)?;
        self.publish_scene_inspection_resync();
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
        Ok(())
    }

    pub fn clear_project(
        &self,
        welcome: WelcomePaneSnapshot,
    ) -> Result<(), EditorStateOperationError> {
        self.shell().lock().state.clear_project(welcome)?;
        self.publish_scene_inspection_resync();
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
        Ok(())
    }

    pub fn import_mesh_asset(
        &self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
        display_path: impl Into<String>,
    ) -> Result<bool, EditorStateOperationError> {
        let changed = self
            .shell()
            .lock()
            .state
            .import_mesh_asset(model, material, display_path)?;
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
        Ok(changed)
    }

    pub fn asset_type_definition(
        &self,
        asset_type: &AssetTypeId,
    ) -> Result<Option<AssetTypeDefinition>, EditorExtensionRegistryError> {
        let mut inner = self.shell().lock();
        Ok(enabled_asset_types_for_shell(&mut inner)?
            .get(asset_type)
            .cloned())
    }

    pub(crate) fn project_asset_type_registry_for_shell(
        shell: &mut crate::ui::workbench::shell_state::WorkbenchShellStateData,
        snapshot: &mut EditorDataSnapshot,
    ) -> Result<(), EditorExtensionRegistryError> {
        let registry = enabled_asset_types_for_shell(shell)?;
        let activity_source = snapshot.asset_activity.visible_assets.clone();
        let projected = projected_asset_items(shell, &activity_source, &registry);
        snapshot.asset_activity.visible_assets = projected.clone();

        let browser_source = snapshot.asset_browser.visible_assets.clone();
        snapshot.asset_browser.visible_assets =
            if browser_source.shares_items_with(&activity_source) {
                projected
            } else {
                projected_asset_items(shell, &browser_source, &registry)
            };
        project_asset_workspace(&mut snapshot.asset_activity, &registry);
        project_asset_workspace(&mut snapshot.asset_browser, &registry);
        Ok(())
    }

    pub(crate) fn present_asset_type_registry_projection_error(
        snapshot: &mut EditorDataSnapshot,
        error: EditorExtensionRegistryError,
    ) {
        // Snapshot projection is a terminal UI boundary. Preserve a readable diagnostic here
        // while all executable host queries retain the original typed error.
        snapshot.status_line = format!("Asset type registry unavailable: {error}");
    }

    #[cfg(test)]
    pub(crate) fn asset_type_registry_cache_counts(&self) -> (u64, u64) {
        self.shell().lock().asset_type_registry_cache.counts()
    }

    pub fn asset_creation_templates(
        &self,
        asset_type: &AssetTypeId,
    ) -> Result<Vec<AssetCreationTemplateDescriptor>, EditorExtensionRegistryError> {
        Ok(self
            .asset_type_definition(asset_type)?
            .map(|definition| definition.creation_templates().to_vec())
            .unwrap_or_default())
    }

    pub fn asset_context_commands(
        &self,
        asset_type: &AssetTypeId,
    ) -> Result<Vec<AssetContextCommandDescriptor>, EditorExtensionRegistryError> {
        Ok(self
            .asset_type_definition(asset_type)?
            .map(|definition| definition.context_commands().to_vec())
            .unwrap_or_default())
    }

    pub fn invoke_asset_creation_template(
        &self,
        source: EditorOperationSource,
        asset_type: &AssetTypeId,
        template_id: &str,
        target_folder: &str,
    ) -> Result<EditorEventRecord, EditorAssetOperationInvokeError> {
        let template = self
            .asset_type_definition(asset_type)?
            .into_iter()
            .flat_map(|definition| definition.creation_templates().to_vec())
            .into_iter()
            .find(|template| template.id() == template_id)
            .ok_or_else(
                || EditorAssetOperationInvokeError::CreationTemplateNotRegistered {
                    asset_type: asset_type.clone(),
                    template_id: template_id.to_string(),
                },
            )?;
        self.invoke_operation(
            source,
            EditorOperationInvocation::new(template.operation().clone()).with_arguments(
                serde_json::json!({
                    "asset_type": asset_type.as_str(),
                    "template_id": template.id(),
                    "target_folder": target_folder,
                    "default_document": template.default_document(),
                }),
            ),
        )
        .map_err(EditorAssetOperationInvokeError::from)
    }

    pub fn invoke_asset_context_command(
        &self,
        source: EditorOperationSource,
        asset_type: &AssetTypeId,
        command_id: &str,
        asset_locator: &str,
    ) -> Result<EditorEventRecord, EditorAssetOperationInvokeError> {
        let command = self
            .asset_type_definition(asset_type)?
            .into_iter()
            .flat_map(|definition| definition.context_commands().to_vec())
            .into_iter()
            .find(|command| command.id() == command_id)
            .ok_or_else(
                || EditorAssetOperationInvokeError::ContextCommandNotRegistered {
                    asset_type: asset_type.clone(),
                    command_id: command_id.to_string(),
                },
            )?;
        self.invoke_operation(
            source,
            EditorOperationInvocation::new(command.operation().clone()).with_arguments(
                serde_json::json!({
                    "asset_type": asset_type.as_str(),
                    "command_id": command.id(),
                    "asset_locator": asset_locator,
                }),
            ),
        )
        .map_err(EditorAssetOperationInvokeError::from)
    }
}

fn projected_asset_items(
    shell: &mut crate::ui::workbench::shell_state::WorkbenchShellStateData,
    source: &AssetWorkspaceItemGeneration,
    registry: &Arc<AssetTypeRegistry>,
) -> AssetWorkspaceItemGeneration {
    if let Some(projected) = shell
        .asset_type_registry_cache
        .projected_asset_items(source, registry)
    {
        return projected;
    }

    let previous = shell
        .asset_type_registry_cache
        .previous_asset_item_projection(registry);
    let project = |item: &mut crate::ui::workbench::snapshot::AssetItemSnapshot| {
        let Ok(asset_type) = AssetTypeId::parse(item.asset_type.asset_type_id.clone()) else {
            return;
        };
        if let Some(definition) = registry.get(&asset_type) {
            item.asset_type = AssetTypeProjectionSnapshot::from_definition(definition);
        }
    };
    let projected = match previous {
        Some((previous_source, previous_projected)) => {
            source.project_items_reusing(&previous_source, &previous_projected, project)
        }
        None => source.project_items(project),
    };
    shell.asset_type_registry_cache.store_projected_asset_items(
        source.clone(),
        Arc::clone(registry),
        projected.clone(),
    );
    projected
}

fn project_asset_workspace(snapshot: &mut AssetWorkspaceSnapshot, registry: &AssetTypeRegistry) {
    snapshot.creation_menu = registry.creation_menu_generation();

    let selection_type = snapshot.selection.asset_type.asset_type_id.clone();
    let Ok(asset_type) = AssetTypeId::parse(selection_type) else {
        return;
    };
    let Some(definition) = registry.get(&asset_type) else {
        return;
    };
    snapshot.selection.asset_type = AssetTypeProjectionSnapshot::from_definition(definition);
    snapshot.selection.toolkit_view_id = definition
        .toolkit()
        .map(|toolkit| toolkit.view_id().to_owned())
        .unwrap_or_default();
    snapshot.selection.toolkit_open_operation = definition
        .toolkit()
        .map(|toolkit| toolkit.open_operation().to_string())
        .unwrap_or_default();
    snapshot.selection.context_commands = definition
        .context_commands()
        .iter()
        .map(|command| AssetOperationProjectionSnapshot {
            asset_type_id: definition.id().to_string(),
            id: command.id().to_owned(),
            display_name: command.display_name().to_owned(),
            operation_id: command.operation().to_string(),
            icon_name: command.icon_name().map(str::to_owned),
            default_document: None,
        })
        .collect();
    for reference in snapshot
        .selection
        .references
        .iter_mut()
        .chain(snapshot.selection.used_by.iter_mut())
    {
        reference.asset_type = reference
            .kind
            .and_then(|kind| projected_asset_type_for_kind(registry, kind));
    }
    for subasset in &mut snapshot.selection.subassets {
        if let Some(asset_type) = projected_asset_type_for_kind(registry, subasset.kind) {
            subasset.asset_type = asset_type;
        }
    }
}

fn projected_asset_type_for_kind(
    registry: &AssetTypeRegistry,
    kind: zircon_runtime_interface::resource::ResourceKind,
) -> Option<AssetTypeProjectionSnapshot> {
    registry
        .get(&AssetTypeId::from_resource_kind(kind))
        .map(AssetTypeProjectionSnapshot::from_definition)
}
