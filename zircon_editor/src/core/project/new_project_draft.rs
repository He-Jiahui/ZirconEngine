use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::{validate_project_name, ProjectTemplateId};

use super::ProjectAuthorityError;

/// Authoring request for a new project; validation belongs to ProjectAuthority.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProjectDraft {
    pub project_name: String,
    pub location: String,
    pub template: ProjectTemplateId,
}

impl NewProjectDraft {
    pub fn renderable_empty_default() -> Self {
        Self {
            project_name: "ZirconProject".to_string(),
            location: default_project_location().to_string_lossy().into_owned(),
            template: ProjectTemplateId::RenderableEmpty,
        }
    }

    pub fn project_root(&self) -> Result<PathBuf, ProjectAuthorityError> {
        validate_project_name(&self.project_name)?;
        let project_name = self.project_name.as_str();
        let location = self.location.trim();
        if location.is_empty() {
            return Err(ProjectAuthorityError::EmptyProjectLocation);
        }
        Ok(PathBuf::from(location).join(project_name))
    }

    pub fn validate_for_creation(&self) -> Result<PathBuf, ProjectAuthorityError> {
        let root = super::filesystem::resolve_project_path(&self.project_root()?)?;
        super::filesystem::validate_creation_target(&root)?;
        Ok(root)
    }
}

fn default_project_location() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(home) = std::env::var_os("USERPROFILE") {
        return default_windows_project_location(home);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("ZirconProjects");
    }
    // Keep the fallback unresolved so the shared project-path resolver owns current-directory
    // resolution and Windows path identity rules.
    PathBuf::from(".")
}

#[cfg(any(target_os = "windows", test))]
fn default_windows_project_location(home: impl Into<PathBuf>) -> PathBuf {
    let mut location = home.into();
    location.reserve("Documents".len() + "ZirconProjects".len() + 2);
    location.push("Documents");
    location.push("ZirconProjects");
    location
}

#[cfg(test)]
#[path = "tests/new_project_draft.rs"]
mod tests;
