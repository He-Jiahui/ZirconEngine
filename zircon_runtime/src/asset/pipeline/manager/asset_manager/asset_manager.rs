use crate::core::framework::channel::{ChannelReceiver, ChannelWakeCallback};
use crate::core::CoreError;

use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatchError};
use crate::asset::{
    AssetImportError, AssetImporterCapabilityReport, AssetImporterHandler, AssetPipelineInfo,
    AssetStatusRecord, AssetUri, AssetUuid, ProjectImportReceipt, ProjectInfo,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait AssetManager: Send + Sync {
    fn pipeline_info(&self) -> AssetPipelineInfo;
    fn register_asset_importer(
        &self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), CoreError>;
    fn asset_importer_capability_reports(&self) -> Vec<AssetImporterCapabilityReport>;
    fn asset_importer_capability_report_for_source(
        &self,
        source_path: &str,
    ) -> Result<AssetImporterCapabilityReport, AssetImportError>;
    fn open_project(&self, root_path: &str) -> Result<ProjectInfo, CoreError>;
    fn open_prepared_project(&self, project: ProjectManager) -> Result<ProjectInfo, CoreError>;
    /// Retires the active project generation.
    ///
    /// `None` means no project was active and no observable asset state changed. `Some(root)` is
    /// returned only after the project snapshot, watchers, source-path index, and project-owned
    /// resources have been retired.
    fn close_project(&self) -> Result<Option<PathBuf>, CoreError>;
    fn current_project_snapshot(&self) -> Option<ProjectManager>;
    fn current_project_source_path(
        &self,
        uri: &AssetUri,
    ) -> Result<Option<PathBuf>, AssetImportError>;
    fn current_project_asset_uris(&self) -> Vec<AssetUri>;
    fn current_project(&self) -> Option<ProjectInfo>;
    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord>;
    fn list_assets(&self) -> Vec<AssetStatusRecord>;
    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange>;
    fn subscribe_asset_changes_with_wake(
        &self,
        _wake: ChannelWakeCallback,
    ) -> ChannelReceiver<AssetChange> {
        self.subscribe_asset_changes()
    }
    fn subscribe_asset_watch_errors(&self) -> ChannelReceiver<AssetWatchError>;
    fn relocate_project_source(
        &self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<Vec<AssetStatusRecord>, CoreError>;
    /// Imports one model source through the Runtime-owned compound project transaction.
    fn import_model_source(&self, source_path: &Path) -> Result<ProjectImportReceipt, CoreError>;
    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError>;
    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError>;
}
