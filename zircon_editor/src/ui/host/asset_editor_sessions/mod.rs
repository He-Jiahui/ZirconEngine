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
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError};

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
