use std::path::PathBuf;

use crate::assets::discover_asset_catalog_for_scope;
use crate::error::HubError;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_asset_catalog(&mut self) -> Result<(), HubError> {
        self.asset_catalog = discover_asset_catalog_for_scope(
            self.selected_project_catalog_root(),
            project_asset_roots(&self.config.recent_projects),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }
}

fn project_asset_roots(projects: &[crate::projects::RecentProject]) -> Vec<PathBuf> {
    projects
        .iter()
        .map(|project| project.path.clone())
        .collect()
}
