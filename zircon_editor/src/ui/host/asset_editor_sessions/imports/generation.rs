use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ui::host::EditorError;

use super::parsed_document::ParsedUiAssetImportDocument;

/// One worker generation's physical parse cache. Logical fragment aliases and
/// multiple open documents share the same physical read/parse result.
#[derive(Default)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetImportGeneration {
    parsed_by_physical_path: BTreeMap<PathBuf, Result<Arc<ParsedUiAssetImportDocument>, String>>,
}

impl UiAssetImportGeneration {
    pub(in crate::ui::host::asset_editor_sessions) fn load_physical_document(
        &mut self,
        physical_path: &Path,
        load: impl FnOnce() -> Result<ParsedUiAssetImportDocument, String>,
    ) -> Result<Arc<ParsedUiAssetImportDocument>, EditorError> {
        let cached = self
            .parsed_by_physical_path
            .entry(physical_path.to_path_buf())
            .or_insert_with(|| load().map(Arc::new))
            .clone();
        cached.map_err(EditorError::UiAsset)
    }
}
