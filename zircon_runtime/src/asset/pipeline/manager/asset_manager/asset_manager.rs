use crate::core::{ChannelReceiver, CoreError};

use crate::asset::watch::AssetChange;
use crate::asset::{AssetImporterHandler, AssetPipelineInfo, AssetStatusRecord, ProjectInfo};
use std::sync::Arc;

pub trait AssetManager: Send + Sync {
    fn pipeline_info(&self) -> AssetPipelineInfo;
    fn register_asset_importer(
        &self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), CoreError>;
    fn open_project(&self, root_path: &str) -> Result<ProjectInfo, CoreError>;
    fn current_project(&self) -> Option<ProjectInfo>;
    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord>;
    fn list_assets(&self) -> Vec<AssetStatusRecord>;
    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange>;
    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError>;
    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError>;
}
