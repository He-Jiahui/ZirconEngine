use std::sync::Arc;

use super::super::super::EditorAssetDetailsGeneration;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn asset_details_generation(
        &self,
        uuid: &str,
    ) -> Option<Arc<EditorAssetDetailsGeneration>> {
        let state = self.state.read().expect("editor asset state lock poisoned");
        state.catalog_generation.details(uuid)
    }
}
