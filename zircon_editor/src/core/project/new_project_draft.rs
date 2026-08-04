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
        return PathBuf::from(home).join("Documents").join("ZirconProjects");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("ZirconProjects");
    }
    // Keep the fallback unresolved so the shared project-path resolver owns current-directory
    // resolution and Windows path identity rules.
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{NewProjectDraft, NewProjectTemplate};

    #[test]
    fn project_root_leaves_relative_locations_for_the_shared_path_resolver() {
        let draft = NewProjectDraft {
            project_name: "Resolver Owned Project".to_string(),
            location: "relative-project-parent".to_string(),
            template: NewProjectTemplate::RenderableEmpty,
        };

        assert_eq!(
            draft.project_root().unwrap(),
            PathBuf::from("relative-project-parent").join("Resolver Owned Project")
        );
    }

    #[cfg(windows)]
    #[test]
    fn drive_relative_creation_location_is_rejected_by_the_shared_path_resolver() {
        let draft = NewProjectDraft {
            project_name: "Resolver Owned Project".to_string(),
            location: r"C:ambiguous-project-parent".to_string(),
            template: NewProjectTemplate::RenderableEmpty,
        };

        assert!(matches!(
            draft.validate_for_creation(),
            Err(super::ProjectAuthorityError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidInput
        ));
    }

    #[test]
    fn default_location_does_not_hide_current_directory_errors() {
        let source = include_str!("new_project_draft.rs");
        let swallowed_current_directory = ["current_dir()", ".unwrap_or_else"].concat();

        assert!(!source.contains(&swallowed_current_directory));
    }
}
