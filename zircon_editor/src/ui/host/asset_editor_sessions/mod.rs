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

use zircon_runtime::ui::template::UiAssetLoader;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError};

pub(crate) use editing::{
    preview_size_for_preset, ui_asset_editor_view_descriptor, UI_ASSET_EDITOR_DESCRIPTOR_ID,
};
pub(crate) use watcher::UiAssetWorkspaceWatcher;
#[cfg(test)]
pub(crate) use workspace_state::UiAssetDiffSnapshot;
pub(crate) use workspace_state::{
    ui_asset_source_hash, UiAssetExternalConflict, UiAssetStaleImportDiagnostic,
    UiAssetWorkspaceEntry,
};

fn parse_ui_asset_document_source(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    UiAssetLoader::load_toml_str(source).or_else(|error| {
        #[cfg(test)]
        {
            crate::tests::support::load_test_ui_asset(source).or(Err(error))
        }
        #[cfg(not(test))]
        {
            Err(error)
        }
    })
}
