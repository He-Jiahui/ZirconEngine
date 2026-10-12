use std::path::PathBuf;
use std::sync::Arc;

use zircon_runtime::asset::watch::AssetChange;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::channel::ChannelWakeCallback;
use zircon_runtime::core::CoreError;

use crate::core::asset::{
    AssetDeletePreflight, AssetSourceWritePolicy, EditorAssetDeletionTicket,
    EditorAssetRelocationTicket, EditorModelImportTicket,
};

use super::super::super::{
    EditorAssetCatalogGeneration, EditorAssetChangeSubscription, EditorAssetDetailsGeneration,
    EditorAssetManager,
};
use super::{
    editor_asset_error::editor_asset_error, parse_uuid::parse_uuid, DefaultEditorAssetManager,
};
impl EditorAssetManager for DefaultEditorAssetManager {
    fn refresh_from_runtime_project(&self) -> Result<(), CoreError> {
        DefaultEditorAssetManager::refresh_from_runtime_project(self)
    }

    fn project_runtime_asset_changes(&self, changes: &[AssetChange]) {
        DefaultEditorAssetManager::project_runtime_asset_changes(self, changes)
    }

    fn deactivate_runtime_project(&self) -> bool {
        DefaultEditorAssetManager::deactivate_runtime_project(self)
    }

    fn catalog_snapshot(&self) -> Arc<EditorAssetCatalogGeneration> {
        self.catalog_snapshot_record()
    }

    fn asset_details(&self, uuid: &str) -> Option<Arc<EditorAssetDetailsGeneration>> {
        self.asset_details_generation(uuid)
    }

    fn asset_delete_preflight(
        &self,
        uuid: &str,
        write_policy: AssetSourceWritePolicy,
    ) -> Result<AssetDeletePreflight, CoreError> {
        DefaultEditorAssetManager::asset_delete_preflight(self, parse_uuid(uuid)?, write_policy)
    }

    fn submit_project_source_deletion(
        &self,
        uuid: &str,
    ) -> Result<EditorAssetDeletionTicket, CoreError> {
        DefaultEditorAssetManager::submit_project_source_deletion(self, parse_uuid(uuid)?)
    }

    fn submit_project_source_relocation(
        &self,
        uuid: &str,
        target: AssetUri,
    ) -> Result<EditorAssetRelocationTicket, CoreError> {
        DefaultEditorAssetManager::submit_project_source_relocation(self, parse_uuid(uuid)?, target)
    }

    fn submit_model_import(
        &self,
        source_path: PathBuf,
    ) -> Result<EditorModelImportTicket, CoreError> {
        DefaultEditorAssetManager::submit_model_import(self, source_path)
    }

    fn subscribe_editor_asset_changes(&self) -> EditorAssetChangeSubscription {
        self.subscribe_editor_asset_changes_impl()
    }

    fn subscribe_editor_asset_changes_with_wake(
        &self,
        wake: ChannelWakeCallback,
    ) -> EditorAssetChangeSubscription {
        self.subscribe_editor_asset_changes_with_wake_impl(wake)
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
