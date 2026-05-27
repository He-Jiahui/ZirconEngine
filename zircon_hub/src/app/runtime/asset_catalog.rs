use std::path::PathBuf;

use crate::assets::discover_asset_catalog_for_scope;
use crate::error::HubError;

use super::{root_paths::push_development_roots, HubRuntime};

impl HubRuntime {
    pub(super) fn refresh_asset_catalog(&mut self) -> Result<(), HubError> {
        self.asset_catalog = discover_asset_catalog_for_scope(
            self.selected_project_path.clone(),
            project_asset_roots(&self.config.recent_projects),
            asset_repo_roots(self.config.settings.default_source_dir.clone()),
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

fn asset_repo_roots(source_dir: PathBuf) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    push_development_roots(&mut roots, source_dir);
    roots
}
