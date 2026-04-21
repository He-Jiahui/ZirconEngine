mod editing;
mod hydration;
mod imports;
mod lifecycle;
mod open;
mod preview_refresh;
mod save;
mod sync;

use zircon_runtime::ui::template::{UiAssetDocument, UiAssetError, UiAssetLoader};

pub(crate) use editing::{
    preview_size_for_preset, ui_asset_editor_view_descriptor, UiAssetWorkspaceEntry,
    UI_ASSET_EDITOR_DESCRIPTOR_ID,
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
