use std::sync::Arc;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord,
};

pub(in crate::ui::host::editor_asset_manager::manager) fn update_asset_in_catalog_generation(
    current: &Arc<EditorAssetCatalogGeneration>,
    updated: EditorAssetCatalogRecord,
    publish_epoch: u64,
) -> Arc<EditorAssetCatalogGeneration> {
    current
        .updated_asset(Arc::new(updated), publish_epoch)
        .map(Arc::new)
        .unwrap_or_else(|| Arc::clone(current))
}
