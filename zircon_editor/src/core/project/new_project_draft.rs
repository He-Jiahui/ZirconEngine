use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::validate_project_name;

use super::{NewProjectTemplate, ProjectAuthorityError};

/// Authoring request for a new project; validation belongs to ProjectAuthority.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProjectDraft {
    pub project_name: String,
    pub location: String,
    pub template: NewProjectTemplate,
}

impl NewProjectDraft {
    pub fn renderable_empty_default() -> Self {
        Self {
            project_name: "ZirconProject".to_string(),
            location: default_project_location().to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        }
    }

    pub fn project_root(&self) -> Result<PathBuf, ProjectAuthorityError> {
        validate_project_name(&self.project_name)?;
        let project_name = self.project_name.as_str();
        let location = self.location.trim();
        if location.is_empty() {
            return Err(ProjectAuthorityError::EmptyProjectLocation);
        }
        let location = PathBuf::from(location);
        let location = if location.is_absolute() {
            location
        } else {
            std::env::current_dir()
                .map_err(|source| ProjectAuthorityError::CurrentDirectory { source })?
                .join(location)
        };
        Ok(location.join(project_name))
    }

    pub fn validate_for_creation(&self) -> Result<PathBuf, ProjectAuthorityError> {
        let root = self.project_root()?;
        super::filesystem::validate_creation_target(&root)?;
        Ok(root)
    }
}

fn default_project_location() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(home) = std::env::var_os("USERPROFILE") {
        return PathBuf::from(home).join("Documents").join("ZirconProjects");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("ZirconProjects");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
