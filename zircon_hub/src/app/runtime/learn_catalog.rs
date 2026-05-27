use std::path::PathBuf;

use crate::error::HubError;
use crate::learn::discover_learn_catalog_for_scope;
use crate::process::{open_folder, OpenFolderCommand};
use crate::state::TaskStatus;

use super::{root_paths::push_development_roots, HubRuntime};

impl HubRuntime {
    pub(super) fn refresh_learn_catalog(&mut self) -> Result<(), HubError> {
        self.learn_catalog = discover_learn_catalog_for_scope(
            self.selected_project_path.clone(),
            learn_catalog_roots(self.config.settings.default_source_dir.clone()),
        )?;
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
    push_development_roots(&mut roots, source_dir);
    roots
}
