use std::sync::Arc;

use super::super::super::EditorAssetCatalogGeneration;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn catalog_snapshot_record(&self) -> Arc<EditorAssetCatalogGeneration> {
        let state = self.read_state_recovering_poison();
        Arc::clone(&state.catalog_generation)
    }
}
