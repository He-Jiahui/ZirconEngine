use zircon_core::{ChannelReceiver, CoreError};

use super::{EditorAssetCatalogSnapshotRecord, EditorAssetChangeRecord, EditorAssetDetailsRecord};

pub trait EditorAssetManager: Send + Sync {
    fn catalog_snapshot(&self) -> EditorAssetCatalogSnapshotRecord;
    fn asset_details(&self, uuid: &str) -> Option<EditorAssetDetailsRecord>;
    fn subscribe_editor_asset_changes(&self) -> ChannelReceiver<EditorAssetChangeRecord>;
    fn mark_preview_dirty(&self, uuid: &str)
        -> Result<Option<EditorAssetDetailsRecord>, CoreError>;
    fn request_preview_refresh(
        &self,
        uuid: &str,
        visible: bool,
    ) -> Result<Option<EditorAssetDetailsRecord>, CoreError>;
}
