use std::path::PathBuf;

use crate::error::HubError;
use crate::learn::discover_learn_catalog;
use crate::process::{open_folder, OpenFolderCommand};
use crate::state::TaskStatus;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_learn_catalog(&mut self) -> Result<(), HubError> {
        self.learn_catalog = discover_learn_catalog(learn_catalog_roots(
            self.config.settings.default_source_dir.clone(),
        ))?;
        Ok(())
    }

    pub(super) fn open_learn_resource(&mut self, resource_path: &str) -> Result<(), HubError> {
        let path = PathBuf::from(resource_path);
        let command = OpenFolderCommand::new(path.clone());
        open_folder(&command)?;
        self.task_status = TaskStatus {
            label: "Resource opened".to_string(),
            detail: path.to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
    }
}

fn learn_catalog_roots(source_dir: PathBuf) -> Vec<PathBuf> {
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
