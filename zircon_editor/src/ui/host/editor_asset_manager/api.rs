use std::sync::Arc;

use zircon_runtime::core::framework::channel::ChannelWakeCallback;
use zircon_runtime::core::CoreError;

use super::{
    EditorAssetCatalogGeneration, EditorAssetChangeSubscription, EditorAssetDetailsGeneration,
};

pub trait EditorAssetManager: Send + Sync {
    fn refresh_from_runtime_project(&self) -> Result<(), CoreError>;
    /// Clears the projection of the retired runtime project.
    ///
    /// Returns `true` only when an active project projection was replaced with a newer empty
    /// generation. `false` is a no-op and emits no catalog change.
    fn deactivate_runtime_project(&self) -> Result<bool, CoreError>;
    fn catalog_snapshot(&self) -> Arc<EditorAssetCatalogGeneration>;
    fn asset_details(&self, uuid: &str) -> Option<Arc<EditorAssetDetailsGeneration>>;
    fn subscribe_editor_asset_changes(&self) -> EditorAssetChangeSubscription;
    fn subscribe_editor_asset_changes_with_wake(
        &self,
        _wake: ChannelWakeCallback,
    ) -> EditorAssetChangeSubscription {
        self.subscribe_editor_asset_changes()
    }
    fn request_preview_refresh(
        &self,
        uuid: &str,
        visible: bool,
    ) -> Result<Option<Arc<EditorAssetDetailsGeneration>>, CoreError>;
}
