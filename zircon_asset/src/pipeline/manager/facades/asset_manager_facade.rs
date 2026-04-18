use crossbeam_channel::unbounded;
use zircon_core::{ChannelReceiver, CoreError};

use super::super::errors::asset_error;
use super::super::project_asset_manager::ProjectAssetManager;
use super::super::records::{build_project_info, build_status_record};
use super::super::resource_sync::{clear_removed_project_resources, project_locators};
use super::super::{
    AssetManager as AssetManagerFacade, AssetPipelineInfo, AssetStatusRecord, ProjectInfo,
};
use crate::{AssetChange, AssetChangeKind, AssetUri, ProjectManager};

impl AssetManagerFacade for ProjectAssetManager {
    fn pipeline_info(&self) -> AssetPipelineInfo {
        AssetPipelineInfo {
            default_worker_count: self.default_worker_count(),
        }
    }

    fn open_project(&self, root_path: &str) -> Result<ProjectInfo, CoreError> {
        let mut project = ProjectManager::open(root_path).map_err(asset_error)?;
        let previous_locators = self
            .project_read()
            .as_ref()
            .map(project_locators)
            .unwrap_or_default();
        let imported = project.scan_and_import().map_err(asset_error)?;
        clear_removed_project_resources(&self.resource_manager(), &previous_locators, &project);
        self.sync_project_resources(&project)?;
        let info = build_project_info(&project);
        self.editor_asset_manager()
            .sync_from_project(project.clone())
            .map_err(asset_error)?;
        *self.project_write() = Some(project);
        self.restart_watcher()?;
        self.broadcast(
            imported
                .into_iter()
                .map(|metadata| {
                    AssetChange::new(
                        AssetChangeKind::Added,
                        metadata.primary_locator().clone(),
                        None,
                    )
                })
                .collect(),
        );
        Ok(info)
    }

    fn current_project(&self) -> Option<ProjectInfo> {
        self.project_read().as_ref().map(build_project_info)
    }

    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord> {
        let uri = AssetUri::parse(uri).ok()?;
        let project = self.project_read();
        let project = project.as_ref()?;
        project
            .registry()
            .get_by_locator(&uri)
            .map(build_status_record)
    }

    fn list_assets(&self) -> Vec<AssetStatusRecord> {
        let project = self.project_read();
        let Some(project) = project.as_ref() else {
            return Vec::new();
        };
        let mut assets = project
            .registry()
            .values()
            .map(build_status_record)
            .collect::<Vec<_>>();
        assets.sort_by(|left, right| left.uri.cmp(&right.uri));
        assets
    }

    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange> {
        let (sender, receiver) = unbounded();
        self.change_subscribers
            .lock()
            .expect("asset subscribers lock poisoned")
            .push(sender);
        receiver
    }

    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError> {
        let uri = AssetUri::parse(uri).map_err(asset_error)?;
        let mut project = self.project_write();
        let Some(project) = project.as_mut() else {
            return Ok(None);
        };
        project.scan_and_import().map_err(asset_error)?;
        self.sync_project_resources(project)?;
        self.editor_asset_manager()
            .sync_from_project(project.clone())
            .map_err(asset_error)?;
        let status = project
            .registry()
            .get_by_locator(&uri)
            .map(build_status_record);
        if status.is_some() {
            self.broadcast(vec![AssetChange::new(AssetChangeKind::Modified, uri, None)]);
        }
        Ok(status)
    }

    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let mut project = self.project_write();
        let Some(project) = project.as_mut() else {
            return Ok(Vec::new());
        };
        let imported = project.scan_and_import().map_err(asset_error)?;
        self.sync_project_resources(project)?;
        self.editor_asset_manager()
            .sync_from_project(project.clone())
            .map_err(asset_error)?;
        let statuses = imported.iter().map(build_status_record).collect::<Vec<_>>();
        self.broadcast(
            imported
                .into_iter()
                .map(|metadata| {
                    AssetChange::new(
                        AssetChangeKind::Modified,
                        metadata.primary_locator().clone(),
                        None,
                    )
                })
                .collect(),
        );
        Ok(statuses)
    }
}
