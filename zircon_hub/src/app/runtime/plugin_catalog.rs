use std::path::PathBuf;

use crate::error::HubError;
use crate::plugins::discover_plugin_catalog;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_plugin_catalog(&mut self) -> Result<(), HubError> {
        self.plugin_catalog = discover_plugin_catalog(plugin_catalog_roots(
            self.config.settings.default_source_dir.clone(),
        ))?;
        Ok(())
    }
}

fn plugin_catalog_roots(source_dir: PathBuf) -> Vec<PathBuf> {
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
    if path.as_os_str().is_empty() {
        return;
    }
    if roots.iter().any(|root| root == &path) {
        return;
    }
    roots.push(path);
}

fn compiled_repo_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
}
