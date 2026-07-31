use std::sync::Arc;

use super::super::super::EditorAssetCatalogGeneration;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn catalog_snapshot_record(&self) -> Arc<EditorAssetCatalogGeneration> {
        let state = self.state.read().expect("editor asset state lock poisoned");
        Arc::clone(&state.catalog_generation)
    }
}
