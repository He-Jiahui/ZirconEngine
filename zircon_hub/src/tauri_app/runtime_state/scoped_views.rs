use std::path::PathBuf;

use crate::assets::discover_asset_catalog_for_scope;
use crate::error::HubError;
use crate::learn::discover_learn_catalog_for_scope;
use crate::plugins::discover_plugin_catalog_with_project_roots;
use crate::projects::project_filesystem_path_key;
use crate::team::discover_team_overview;

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn refresh_source_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }

    pub(super) fn refresh_selected_project_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }

    fn refresh_asset_catalog(&mut self) -> Result<(), HubError> {
        self.asset_catalog = discover_asset_catalog_for_scope(
            self.selected_project_catalog_root(),
            self.config
                .recent_projects
                .iter()
                .map(|project| project.path.clone())
                .collect::<Vec<_>>(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    fn refresh_learn_catalog(&mut self) -> Result<(), HubError> {
        self.learn_catalog = discover_learn_catalog_for_scope(
            self.selected_project_catalog_root(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    fn refresh_plugin_catalog(&mut self) -> Result<(), HubError> {
        self.plugin_catalog = discover_plugin_catalog_with_project_roots(
            self.selected_project_catalog_root().into_iter(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    fn refresh_team_overview(&mut self) -> Result<(), HubError> {
        let mut roots = Vec::new();
        if let Some(project_root) = self.selected_project_catalog_root() {
            push_unique_root(&mut roots, project_root);
        }
        for source_root in self.source_engine_catalog_roots() {
            push_unique_root(&mut roots, source_root);
        }
        self.team_overview = discover_team_overview(roots)?;
        Ok(())
    }

    fn selected_project_catalog_root(&self) -> Option<PathBuf> {
        self.snapshot()
            .scope()
            .selected_project()
            .map(|project| project.path.clone())
    }

    fn source_engine_catalog_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        let scope = self.snapshot().scope();
        let Some(engine_id) = scope.source_engine.engine_id() else {
            return roots;
        };
        if let Some(engine) = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
        {
            push_development_roots(&mut roots, engine.source_dir.clone());
        }
        roots
    }
}

fn push_unique_root(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    let candidate_key = project_filesystem_path_key(&path);
    if roots
        .iter()
        .any(|root| project_filesystem_path_key(root) == candidate_key)
    {
        return;
    }
    roots.push(path);
}

fn push_development_roots(roots: &mut Vec<PathBuf>, source_dir: PathBuf) {
    push_unique_root(roots, source_dir);
    if let Ok(current_dir) = std::env::current_dir() {
        push_unique_root(roots, current_dir);
    }
    if let Some(compiled_repo_root) = compiled_repo_root() {
        push_unique_root(roots, compiled_repo_root);
    }
}

fn compiled_repo_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
}

#[cfg(test)]
#[path = "tests/scoped_views.rs"]
mod tests;
