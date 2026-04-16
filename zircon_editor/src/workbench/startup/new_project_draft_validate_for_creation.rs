use std::path::PathBuf;

use super::new_project_draft::NewProjectDraft;

impl NewProjectDraft {
    pub fn validate_for_creation(&self) -> Result<PathBuf, String> {
        let root = self.project_root()?;
        if root.exists() {
            if root.is_file() {
                return Err("Target path already exists as a file".to_string());
            }
            let mut entries = root.read_dir().map_err(|error| error.to_string())?;
            if entries
                .next()
                .transpose()
                .map_err(|error| error.to_string())?
                .is_some()
            {
                return Err("Target directory must be empty".to_string());
            }
        }
        Ok(root)
    }
}
