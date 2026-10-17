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

use super::{
    EditorAssetCatalogGeneration, EditorAssetChangeSubscription, EditorAssetDetailsGeneration,
};

pub trait EditorAssetManager: Send + Sync {
    fn refresh_from_runtime_project(&self) -> Result<(), CoreError>;

    /// Custom managers must explicitly implement receipt inclusion. Unsupported
    /// is a repair outcome; it never acknowledges a durable receipt.
    fn model_import_projection(
        &self,
        _activation: &zircon_runtime::asset::project::ProjectActivationIdentity,
        _receipt: &zircon_runtime::asset::ProjectImportReceipt,
    ) -> EditorModelImportProjection {
        EditorModelImportProjection::Unsupported
    }

    /// Projects Runtime watcher changes into editor-local stale state before the next catalog
    /// refresh observes a committed Runtime input generation.
    fn project_runtime_asset_changes(&self, changes: &[std::sync::Arc<AssetChange>]);
    /// Clears the projection of the retired runtime project.
    ///
    /// Returns `true` only when an active project projection was replaced with a newer empty
    /// generation. `false` emits no catalog change, but still retires registered source-sync work.
    fn deactivate_runtime_project(&self) -> bool;
    fn catalog_snapshot(&self) -> Arc<EditorAssetCatalogGeneration>;
    fn asset_details(&self, uuid: &str) -> Option<Arc<EditorAssetDetailsGeneration>>;
    /// Returns the runtime-registry delete admission for an asset without mutating source files.
    fn asset_delete_preflight(
        &self,
        uuid: &str,
        write_policy: AssetSourceWritePolicy,
    ) -> Result<AssetDeletePreflight, CoreError>;
    /// Queues a Runtime-owned source deletion after the caller accepts the current preflight.
    fn submit_project_source_deletion(
        &self,
        uuid: &str,
    ) -> Result<EditorAssetDeletionTicket, CoreError>;
    /// Queues a Runtime-owned source relocation. The caller polls the ticket and installs the
    /// already-published editor catalog generation after durable commit.
    fn submit_project_source_relocation(
        &self,
        uuid: &str,
        target: AssetUri,
    ) -> Result<EditorAssetRelocationTicket, CoreError>;
    /// Queues one Runtime-owned compound model import and returns its completion receipt ticket.
    fn submit_model_import(
        &self,
        source_path: PathBuf,
    ) -> Result<EditorModelImportTicket, CoreError>;
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

/// The built-in projection proves an already committed model receipt against one
/// atomically published project/index pair. A higher sequence alone is not proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorModelImportProjection {
    AwaitingPublication,
    Published { generation_sequence: u64 },
    Superseded,
    Unsupported,
}

