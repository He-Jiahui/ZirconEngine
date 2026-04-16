use crate::editor_event::{
    EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode,
    EditorEventEffect,
};
use crate::snapshot::{
    AssetUtilityTab as SnapshotAssetUtilityTab, AssetViewMode as SnapshotAssetViewMode,
};

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;
use super::common::{asset_effects, open_view, parse_asset_kind_filter};

pub(super) fn execute_asset_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorAssetEvent,
) -> Result<ExecutionOutcome, String> {
    match event {
        EditorAssetEvent::OpenAsset { asset_path } => {
            if !asset_path.ends_with(".ui.toml") {
                inner
                    .state
                    .set_status_line(format!("Open asset requested for {asset_path}"));
                return Ok(asset_effects(false, false, false));
            }
            let instance_id = inner
                .manager
                .open_ui_asset_editor_by_id(asset_path, None)
                .map_err(|error| error.to_string())?;
            let focused = inner
                .manager
                .focus_view(&instance_id)
                .map_err(|error| error.to_string())?;
            inner
                .state
                .set_status_line(format!("Opened UI asset editor for {asset_path}"));
            Ok(ExecutionOutcome {
                changed: focused || !instance_id.0.is_empty(),
                effects: vec![
                    EditorEventEffect::LayoutChanged,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        EditorAssetEvent::SelectFolder { folder_id } => {
            inner.state.select_asset_folder(folder_id.clone());
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SelectItem { asset_uuid } => {
            inner.state.select_asset(Some(asset_uuid.clone()));
            Ok(asset_effects(true, true, true))
        }
        EditorAssetEvent::ActivateReference { asset_uuid } => {
            inner.state.navigate_to_asset(asset_uuid);
            Ok(asset_effects(true, true, true))
        }
        EditorAssetEvent::SetSearchQuery { query } => {
            inner.state.set_asset_search_query(query.clone());
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SetKindFilter { kind } => {
            inner
                .state
                .set_asset_kind_filter(parse_asset_kind_filter(kind.as_deref())?);
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::SetViewMode { surface, view_mode } => {
            match (surface, view_mode) {
                (EditorAssetSurface::Activity, EditorAssetViewMode::List) => inner
                    .state
                    .set_asset_activity_view_mode(SnapshotAssetViewMode::List),
                (EditorAssetSurface::Activity, EditorAssetViewMode::Thumbnail) => inner
                    .state
                    .set_asset_activity_view_mode(SnapshotAssetViewMode::Thumbnail),
                (EditorAssetSurface::Browser, EditorAssetViewMode::List) => inner
                    .state
                    .set_asset_browser_view_mode(SnapshotAssetViewMode::List),
                (EditorAssetSurface::Browser, EditorAssetViewMode::Thumbnail) => inner
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
                EditorAssetSurface::Activity => inner.state.set_asset_activity_tab(tab),
                EditorAssetSurface::Browser => inner.state.set_asset_browser_tab(tab),
            }
            Ok(asset_effects(true, false, true))
        }
        EditorAssetEvent::OpenAssetBrowser => {
            let mut outcome = open_view(inner, "editor.asset_browser", "Opened asset browser")?;
            outcome
                .effects
                .push(EditorEventEffect::AssetPreviewRefreshRequested);
            Ok(outcome)
        }
        EditorAssetEvent::LocateSelectedAsset => {
            let mut outcome = open_view(inner, "editor.assets", "Opened assets")?;
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
