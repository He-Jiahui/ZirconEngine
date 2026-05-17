use std::path::PathBuf;

use crate::assets::discover_asset_catalog;
use crate::error::HubError;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_asset_catalog(&mut self) -> Result<(), HubError> {
        self.asset_catalog = discover_asset_catalog(
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
    push_non_empty(&mut roots, source_dir);
    if let Ok(current_dir) = std::env::current_dir() {
        push_non_empty(&mut roots, current_dir);
    }
    if let Some(compiled_repo_root) = compiled_repo_root() {
        push_non_empty(&mut roots, compiled_repo_root);
    }
    roots
}

fn push_non_empty(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() || roots.iter().any(|root| root == &path) {
        return;
    }
    roots.push(path);
}

fn compiled_repo_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
}
