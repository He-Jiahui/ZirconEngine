use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ui::host::EditorError;

use super::parsed_document::ParsedUiAssetImportDocument;

/// One worker generation's physical parse cache. Logical fragment aliases and
/// multiple open documents share the same physical read/parse result.
#[derive(Default)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetImportGeneration {
    parsed_by_physical_path: HashMap<PathBuf, Result<Arc<ParsedUiAssetImportDocument>, String>>,
}

impl UiAssetImportGeneration {
    pub(in crate::ui::host::asset_editor_sessions) fn load_physical_document(
        &mut self,
        physical_path: &Path,
        load: impl FnOnce() -> Result<ParsedUiAssetImportDocument, String>,
    ) -> Result<Arc<ParsedUiAssetImportDocument>, EditorError> {
        if let Some(cached) = self.parsed_by_physical_path.get(physical_path).cloned() {
            return cached.map_err(EditorError::UiAsset);
        }

        let loaded = load().map(Arc::new);
        self.parsed_by_physical_path
            .insert(physical_path.to_path_buf(), loaded.clone());
        loaded.map_err(EditorError::UiAsset)
    }
}

#[cfg(test)]
#[path = "generation/hash_cache_tests.rs"]
mod hash_cache_tests;
