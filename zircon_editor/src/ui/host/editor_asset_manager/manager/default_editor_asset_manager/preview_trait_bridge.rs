use zircon_runtime::core::CoreError;

use super::super::super::{
    EditorAssetCatalogSnapshotRecord, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetManager,
};
use super::{
    editor_asset_error::editor_asset_error, parse_uuid::parse_uuid, DefaultEditorAssetManager,
};
use zircon_runtime::core::ChannelReceiver;

impl EditorAssetManager for DefaultEditorAssetManager {
    fn refresh_from_runtime_project(&self) -> Result<(), CoreError> {
        DefaultEditorAssetManager::refresh_from_runtime_project(self).map_err(editor_asset_error)
    }

    fn catalog_snapshot(&self) -> EditorAssetCatalogSnapshotRecord {
        self.catalog_snapshot_record()
    }

    fn asset_details(&self, uuid: &str) -> Option<EditorAssetDetailsRecord> {
        self.asset_details_record(uuid)
    }

    fn subscribe_editor_asset_changes(&self) -> ChannelReceiver<EditorAssetChangeRecord> {
        self.subscribe_editor_asset_changes_impl()
    }

    fn request_preview_refresh(
        &self,
        uuid: &str,
        visible: bool,
    ) -> Result<Option<EditorAssetDetailsRecord>, CoreError> {
        let asset_uuid = parse_uuid(uuid)?;
        self.request_preview_refresh(asset_uuid, visible)
            .map_err(editor_asset_error)?;
        Ok(self.asset_details_record(uuid))
    }
}
