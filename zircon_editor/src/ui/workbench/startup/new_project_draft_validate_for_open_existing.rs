use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectPaths;

use super::new_project_draft::NewProjectDraft;

impl NewProjectDraft {
    pub fn validate_for_open_existing(&self) -> Result<PathBuf, String> {
        let root = self.project_root()?;
        if !root.exists() {
            return Err("Project directory does not exist".to_string());
        }
        let paths = ProjectPaths::from_root(&root).map_err(|error| error.to_string())?;
        if !paths.manifest_path().exists() {
            return Err("zircon-project.toml not found in target directory".to_string());
        }
        Ok(root)
    }
}
