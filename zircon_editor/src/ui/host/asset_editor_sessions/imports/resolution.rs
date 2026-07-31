use std::collections::BTreeSet;

use super::UiAssetImportDocuments;

#[derive(Default)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetImportResolution {
    pub(in crate::ui::host::asset_editor_sessions) documents: UiAssetImportDocuments,
    pub(in crate::ui::host::asset_editor_sessions) dependencies: BTreeSet<String>,
}
