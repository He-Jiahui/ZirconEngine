use zircon_runtime::asset::AssetUri;

use crate::core::asset::AssetToolkitOpenRoute;
use crate::core::commands::{EditorCommandRegistry, EditorCommandRegistryError};
use crate::core::editor_event::{
    EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode,
    EditorEventEffect,
};
use crate::core::editor_operation::EditorOperationSource;
use crate::ui::host::editor_extension_registration::enabled_asset_types_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::snapshot::{
    AssetUtilityTab as SnapshotAssetUtilityTab, AssetViewMode as SnapshotAssetViewMode,
};
use crate::ui::workbench::view::ViewDescriptorId;

use super::common::{asset_effects, open_view, parse_asset_kind_filter};
use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_asset_event(
    controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    event: &EditorAssetEvent,
) -> Result<ExecutionOutcome, String> {
    match event {
        EditorAssetEvent::OpenAsset { asset_locator } => {
            let asset_locator = match AssetUri::parse(asset_locator) {
                Ok(asset_locator) => asset_locator,
                Err(error) => {
                    shell
                        .state
                        .set_status_line(format!("Invalid asset locator {asset_locator}: {error}"));
                    return Ok(asset_effects(false, false, false));
                }
            };
            let Some(asset_type) = shell.state.asset_type_id_for_locator(&asset_locator) else {
                shell
                    .state
                    .set_status_line(format!("Asset type is not indexed for {asset_locator}"));
                return Ok(asset_effects(false, false, false));
            };
            let registry =
                enabled_asset_types_for_shell(shell).map_err(|error| error.to_string())?;
            let Some(definition) = registry.get(&asset_type) else {
                return Err(format!("asset type `{asset_type}` is not registered"));
            };
            if let Some(toolkit) = definition.toolkit() {
                let operation = {
                    controller
                        .commands()
                        .lock()
                        .command(toolkit.open_operation().as_str())
                        .cloned()
                }
                .ok_or_else(|| {
                    EditorCommandRegistryError::MissingCommand(toolkit.open_operation().clone())
                        .to_string()
                })?;
                let context =
                    controller.command_eval_ctx_for_source(&EditorOperationSource::UiBinding);
                EditorCommandRegistry::ensure_enabled(&operation, &context)
                    .map_err(|error| error.to_string())?;
                return open_asset_document_view(
                    shell,
                    toolkit.view_id(),
                    AssetToolkitOpenRoute::new(asset_locator, toolkit.open_operation().clone()),
                    definition.presentation().display_name(),
                    "Opened asset toolkit for",
                );
            }
            shell
                .state
                .set_status_line(format!("No toolkit is registered for `{asset_type}`"));
            Ok(asset_effects(false, false, false))
        }
        EditorAssetEvent::SelectFolder { folder_id } => {
            shell.state.select_asset_folder(folder_id.clone());
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SelectItem { asset_uuid } => {
            shell.state.select_asset(Some(asset_uuid.clone()));
            Ok(asset_effects(true, true, true))
        }
        EditorAssetEvent::ActivateReference { asset_uuid } => {
            shell.state.navigate_to_asset(asset_uuid);
            Ok(asset_effects(true, true, true))
        }
        EditorAssetEvent::SetSearchQuery { query } => {
            shell.state.set_asset_search_query(query.clone());
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SetKindFilter { kind } => {
            shell
                .state
                .set_asset_kind_filter(parse_asset_kind_filter(kind.as_deref())?);
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SetViewMode { surface, view_mode } => {
            match (surface, view_mode) {
                (EditorAssetSurface::Activity, EditorAssetViewMode::List) => shell
                    .state
                    .set_asset_activity_view_mode(SnapshotAssetViewMode::List),
                (EditorAssetSurface::Activity, EditorAssetViewMode::Thumbnail) => shell
                    .state
                    .set_asset_activity_view_mode(SnapshotAssetViewMode::Thumbnail),
                (EditorAssetSurface::Browser, EditorAssetViewMode::List) => shell
                    .state
                    .set_asset_browser_view_mode(SnapshotAssetViewMode::List),
                (EditorAssetSurface::Browser, EditorAssetViewMode::Thumbnail) => shell
                    .state
                    .set_asset_browser_view_mode(SnapshotAssetViewMode::Thumbnail),
            }
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SetUtilityTab { surface, tab } => {
            let tab = match tab {
                EditorAssetUtilityTab::Preview => SnapshotAssetUtilityTab::Preview,
                EditorAssetUtilityTab::References => SnapshotAssetUtilityTab::References,
                EditorAssetUtilityTab::Metadata => SnapshotAssetUtilityTab::Metadata,
                EditorAssetUtilityTab::Plugins => SnapshotAssetUtilityTab::Plugins,
            };
            match surface {
                EditorAssetSurface::Activity => shell.state.set_asset_activity_tab(tab),
                EditorAssetSurface::Browser => shell.state.set_asset_browser_tab(tab),
            }
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::OpenAssetBrowser => {
            let mut outcome = open_view(shell, "editor.asset_browser", "Opened asset browser")?;
            outcome
                .effects
                .push(EditorEventEffect::AssetPreviewRefreshRequested);
            Ok(outcome)
        }
        EditorAssetEvent::LocateSelectedAsset => {
            let mut outcome = open_view(shell, "editor.assets", "Opened assets")?;
            outcome
                .effects
                .push(EditorEventEffect::AssetPreviewRefreshRequested);
            Ok(outcome)
        }
        EditorAssetEvent::ImportModel => Ok(ExecutionOutcome {
            changed: false,
            effects: vec![EditorEventEffect::ImportModelRequested],
        }),
    }
}

fn open_asset_document_view(
    shell: &mut WorkbenchShellStateData,
    descriptor_id: &str,
    route: AssetToolkitOpenRoute,
    fallback_title: &str,
    status_prefix: &str,
) -> Result<ExecutionOutcome, String> {
    let asset_locator = route.asset_locator().to_string();
    let payload = serde_json::to_value(&route).map_err(|error| error.to_string())?;
    let instance_id = shell
        .manager
        .open_view(ViewDescriptorId::new(descriptor_id), None)
        .map_err(|error| error.to_string())?;
    shell
        .manager
        .update_view_instance_metadata(
            &instance_id,
            Some(asset_document_title(route.asset_locator(), fallback_title)),
            Some(false),
            Some(payload),
        )
        .map_err(|error| error.to_string())?;
    let focused = shell
        .manager
        .focus_view(&instance_id)
        .map_err(|error| error.to_string())?;
    shell
        .state
        .set_status_line(format!("{status_prefix} {asset_locator}"));
    Ok(ExecutionOutcome {
        changed: focused || !instance_id.0.is_empty(),
        effects: vec![
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

fn asset_document_title(asset_locator: &AssetUri, fallback_title: &str) -> String {
    asset_locator
        .path()
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(fallback_title)
        .to_string()
}
