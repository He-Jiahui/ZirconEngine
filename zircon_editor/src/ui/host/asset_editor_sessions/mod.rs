mod editing;
mod hydration;
mod imports;
mod lifecycle;
mod open;
mod preview_refresh;
mod refresh;
mod save;
mod sync;
mod watcher;
mod workspace_state;

use crate::ui::template::EditorTemplateRuntimeService;
use crate::ui::{asset_editor::UiAssetEditorRoute, asset_editor::UiAssetEditorSession};
use zircon_runtime::ui::v2::{UiV2AssetLoader, UiZuiAssetLoader};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind, v2::UiV2AssetKind};

pub(crate) use editing::{
    preview_size_for_preset, ui_asset_editor_view_descriptor, UI_ASSET_EDITOR_DESCRIPTOR_ID,
};
pub(crate) use watcher::UiAssetWorkspaceWatcher;
pub(crate) use workspace_state::UiAssetDiffSnapshot;
pub(crate) use workspace_state::{
    ui_asset_source_hash, UiAssetExternalConflict, UiAssetStaleImportDiagnostic,
    UiAssetWorkspaceEntry,
};

fn parse_ui_asset_document_source(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    EditorTemplateRuntimeService.parse_document_source(source)
}

pub(super) fn build_ui_asset_editor_session_from_source(
    route: UiAssetEditorRoute,
    source: String,
    preview_size: UiSize,
) -> Result<UiAssetEditorSession, String> {
    if is_v2_backed_ui_asset_id(&route.asset_id) {
        UiAssetEditorSession::from_v2_source(route, source, preview_size)
    } else {
        UiAssetEditorSession::from_source(route, source, preview_size)
    }
    .map_err(|error| error.to_string())
}

pub(super) fn ui_asset_editor_route_from_source(
    asset_id: impl Into<String>,
    source: &str,
    mode: crate::ui::asset_editor::UiAssetEditorMode,
) -> Result<UiAssetEditorRoute, String> {
    let asset_id = asset_id.into();
    let asset_kind = if is_v2_backed_ui_asset_id(&asset_id) {
        let document = if is_zui_asset_id(&asset_id) {
            UiZuiAssetLoader::load_zui_str(source).map_err(|error| error.to_string())?
        } else {
            UiV2AssetLoader::load_toml_str(source).map_err(|error| error.to_string())?
        };
        legacy_asset_kind_for_v2(document.asset.kind)
    } else {
        let document = parse_ui_asset_document_source(source).map_err(|error| error.to_string())?;
        document.asset.kind
    };
    Ok(UiAssetEditorRoute::new(asset_id, asset_kind, mode))
}

pub(super) fn is_v2_backed_ui_asset_id(asset_id: &str) -> bool {
    let asset_id = asset_id.trim().to_ascii_lowercase();
    asset_id.ends_with(".zui") || asset_id.ends_with(".v2.ui.toml")
}

fn is_zui_asset_id(asset_id: &str) -> bool {
    asset_id.trim().to_ascii_lowercase().ends_with(".zui")
}

const fn legacy_asset_kind_for_v2(kind: UiV2AssetKind) -> UiAssetKind {
    match kind {
        UiV2AssetKind::View => UiAssetKind::Layout,
        UiV2AssetKind::Component => UiAssetKind::Widget,
        UiV2AssetKind::Style => UiAssetKind::Style,
    }
}
