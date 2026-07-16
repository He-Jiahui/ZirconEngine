use crate::core::framework::channel::ChannelReceiver;
use crate::core::CoreError;

use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatchError};
use crate::asset::{
    AssetImportError, AssetImporterCapabilityReport, AssetImporterHandler, AssetPipelineInfo,
    AssetStatusRecord, ProjectInfo,
};
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
    fn current_project_snapshot(&self) -> Option<ProjectManager>;
    fn current_project(&self) -> Option<ProjectInfo>;
    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord>;
    fn list_assets(&self) -> Vec<AssetStatusRecord>;
    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange>;
    fn subscribe_asset_watch_errors(&self) -> ChannelReceiver<AssetWatchError>;
    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError>;
    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError>;
}
