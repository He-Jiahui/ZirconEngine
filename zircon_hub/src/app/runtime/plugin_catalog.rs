use std::path::PathBuf;

use crate::error::HubError;
use crate::plugins::discover_plugin_catalog_with_project_roots;

use super::{root_paths::push_development_roots, HubRuntime};

impl HubRuntime {
    pub(super) fn refresh_plugin_catalog(&mut self) -> Result<(), HubError> {
        self.plugin_catalog = discover_plugin_catalog_with_project_roots(
            selected_project_plugin_roots(self.selected_project_path.clone()),
            plugin_catalog_roots(self.config.settings.default_source_dir.clone()),
        )?;
        Ok(())
    }
}

fn selected_project_plugin_roots(selected_project_path: Option<PathBuf>) -> Vec<PathBuf> {
    selected_project_path.into_iter().collect()
}

fn plugin_catalog_roots(source_dir: PathBuf) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    push_development_roots(&mut roots, source_dir);
    roots
}
