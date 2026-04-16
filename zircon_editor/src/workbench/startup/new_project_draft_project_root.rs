use std::path::PathBuf;

use super::new_project_draft::NewProjectDraft;

impl NewProjectDraft {
    pub fn project_root(&self) -> Result<PathBuf, String> {
        let project_name = self.project_name.trim();
        if project_name.is_empty() {
            return Err("Project name is required".to_string());
        }

        let location = self.location.trim();
        if location.is_empty() {
            return Err("Location is required".to_string());
        }

        let location = PathBuf::from(location);
        let root = if location.is_absolute() {
            location.join(project_name)
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join(location)
                .join(project_name)
        };
        Ok(root)
    }
}
