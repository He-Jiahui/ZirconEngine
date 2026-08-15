use std::sync::Arc;

use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::core::asset::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetTypeDefinition,
    AssetTypeId, AssetTypeRegistry,
};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editor_event::EditorEventRecord;
use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationSource};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetDetailsGeneration,
};
use crate::ui::host::editor_extension_registration::enabled_asset_types_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::snapshot::{
    AssetOperationProjectionSnapshot, AssetTypeProjectionSnapshot, AssetWorkspaceSnapshot,
    EditorDataSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

impl EditorHostEventController {
    pub fn set_session_mode(&self, session_mode: EditorSessionMode) {
        self.shell().lock().state.set_session_mode(session_mode);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn set_welcome_snapshot(&self, welcome: WelcomePaneSnapshot) {
        self.shell().lock().state.set_welcome_snapshot(welcome);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_catalog(&self, catalog: Arc<EditorAssetCatalogGeneration>) {
        self.shell().lock().state.sync_asset_catalog(catalog);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_resources(
        &self,
        resources: Arc<zircon_runtime::core::resource::ResourceManagementGeneration>,
    ) -> bool {
        let changed = self.shell().lock().state.sync_asset_resources(resources);
        if changed {
            self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
        }
        changed
    }

    pub fn sync_asset_details(&self, details: Option<Arc<EditorAssetDetailsGeneration>>) {
        self.shell().lock().state.sync_asset_details(details);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn replace_world(
        &self,
        world: impl Into<AuthoringWorldSeed>,
        project_path: impl Into<String>,
    ) -> Result<(), String> {
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

    pub fn clear_project(&self, welcome: WelcomePaneSnapshot) -> Result<(), String> {
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
    ) -> Result<bool, String> {
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

    pub fn asset_type_definition(&self, asset_type: &AssetTypeId) -> Option<AssetTypeDefinition> {
        let mut inner = self.shell().lock();
        enabled_asset_types_for_shell(&mut inner)
            .ok()?
            .get(asset_type)
            .cloned()
    }

    pub(crate) fn project_asset_type_registry_for_shell(
        shell: &mut crate::ui::workbench::shell_state::WorkbenchShellStateData,
        snapshot: &mut EditorDataSnapshot,
    ) {
        let Ok(registry) = enabled_asset_types_for_shell(shell) else {
            return;
        };
        project_asset_workspace(&mut snapshot.asset_activity, &registry);
        project_asset_workspace(&mut snapshot.asset_browser, &registry);
    }

    #[cfg(test)]
    pub(crate) fn asset_type_registry_cache_counts(&self) -> (u64, u64) {
        self.shell().lock().asset_type_registry_cache.counts()
    }

    pub fn asset_creation_templates(
        &self,
        asset_type: &AssetTypeId,
    ) -> Vec<AssetCreationTemplateDescriptor> {
        self.asset_type_definition(asset_type)
            .map(|definition| definition.creation_templates().to_vec())
            .unwrap_or_default()
    }

    pub fn asset_context_commands(
        &self,
        asset_type: &AssetTypeId,
    ) -> Vec<AssetContextCommandDescriptor> {
        self.asset_type_definition(asset_type)
            .map(|definition| definition.context_commands().to_vec())
            .unwrap_or_default()
    }

    pub fn invoke_asset_creation_template(
        &self,
        source: EditorOperationSource,
        asset_type: &AssetTypeId,
        template_id: &str,
        target_folder: &str,
    ) -> Result<EditorEventRecord, String> {
        let template = self
            .asset_creation_templates(asset_type)
            .into_iter()
            .find(|template| template.id() == template_id)
            .ok_or_else(|| {
                format!(
                    "asset creation template `{template_id}` is not registered for `{asset_type}`"
                )
            })?;
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
    }

    pub fn invoke_asset_context_command(
        &self,
        source: EditorOperationSource,
        asset_type: &AssetTypeId,
        command_id: &str,
        asset_locator: &str,
    ) -> Result<EditorEventRecord, String> {
        let command = self
            .asset_context_commands(asset_type)
            .into_iter()
            .find(|command| command.id() == command_id)
            .ok_or_else(|| {
                format!("asset context command `{command_id}` is not registered for `{asset_type}`")
            })?;
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
    }
}

fn project_asset_workspace(snapshot: &mut AssetWorkspaceSnapshot, registry: &AssetTypeRegistry) {
    for item in &mut snapshot.visible_assets {
        let Ok(asset_type) = AssetTypeId::parse(item.asset_type.asset_type_id.clone()) else {
            continue;
        };
        if let Some(definition) = registry.get(&asset_type) {
            item.asset_type = AssetTypeProjectionSnapshot::from_definition(definition);
        }
    }

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
