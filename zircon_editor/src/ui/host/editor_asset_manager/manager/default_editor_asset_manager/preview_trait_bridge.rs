use std::sync::Arc;

use zircon_runtime::core::CoreError;

use super::super::super::{
    EditorAssetCatalogGeneration, EditorAssetChangeSubscription, EditorAssetDetailsGeneration,
    EditorAssetManager,
};
use super::{
    DefaultEditorAssetManager, editor_asset_error::editor_asset_error, parse_uuid::parse_uuid,
};
impl EditorAssetManager for DefaultEditorAssetManager {
    fn refresh_from_runtime_project(&self) -> Result<(), CoreError> {
        DefaultEditorAssetManager::refresh_from_runtime_project(self)
    }

    fn deactivate_runtime_project(&self) -> Result<bool, CoreError> {
        DefaultEditorAssetManager::deactivate_runtime_project(self)
    }

    fn catalog_snapshot(&self) -> Arc<EditorAssetCatalogGeneration> {
        self.catalog_snapshot_record()
    }

    fn asset_details(&self, uuid: &str) -> Option<Arc<EditorAssetDetailsGeneration>> {
        self.asset_details_generation(uuid)
    }

    fn subscribe_editor_asset_changes(&self) -> EditorAssetChangeSubscription {
        self.subscribe_editor_asset_changes_impl()
    }

    fn request_preview_refresh(
        &self,
        uuid: &str,
        visible: bool,
    ) -> Result<Option<Arc<EditorAssetDetailsGeneration>>, CoreError> {
        let asset_uuid = parse_uuid(uuid)?;
        self.request_preview_refresh(asset_uuid, visible)
            .map_err(editor_asset_error)?;
        Ok(self.asset_details_generation(uuid))
    }
}
