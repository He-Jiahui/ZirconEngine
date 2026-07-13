use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::project::{
    validate_project_name, ProjectNameError, ProjectTemplateId,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CreateProjectRequestError {
    #[error("project name is invalid: {source}")]
    ProjectName {
        #[from]
        #[source]
        source: ProjectNameError,
    },
    #[error("project location is required")]
    MissingLocation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub project_name: String,
    pub location: PathBuf,
    pub template: ProjectTemplate,
}

impl CreateProjectRequest {
    pub fn new(
        project_name: impl Into<String>,
        location: impl Into<PathBuf>,
        template: ProjectTemplate,
    ) -> Self {
        let project_name = project_name.into();
        Self {
            project_name,
            location: location.into(),
            template,
        }
    }

    pub fn validate_launch_fields(&self) -> Result<(), CreateProjectRequestError> {
        validate_project_name(&self.project_name)?;
        if self.location.as_os_str().is_empty() {
            return Err(CreateProjectRequestError::MissingLocation);
        }
        Ok(())
    }

    pub fn target_root(&self) -> PathBuf {
        self.location.join(&self.project_name)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectTemplate {
    #[default]
    RenderableEmpty,
}

impl ProjectTemplate {
    pub const fn pack_id(self) -> ProjectTemplateId {
        match self {
            Self::RenderableEmpty => ProjectTemplateId::RenderableEmpty,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::RenderableEmpty => "renderable-empty",
        }
    }

    pub fn as_editor_arg(self) -> &'static str {
        match self {
            Self::RenderableEmpty => "renderable-empty",
        }
    }

    pub fn from_enabled_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "renderable-empty" => Some(Self::RenderableEmpty),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectTemplateInfo {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub enabled: bool,
}

pub fn project_template_catalog() -> &'static [ProjectTemplateInfo] {
    &[
        ProjectTemplateInfo {
            id: "renderable-empty",
            title: "Renderable Empty",
            category: "Core",
            description: "Minimal renderable project with the current engine runtime.",
            enabled: true,
        },
        ProjectTemplateInfo {
            id: "2d-scene",
            title: "2D Scene",
            category: "Core",
            description: "Reserved for the 2D renderer workflow.",
            enabled: false,
        },
        ProjectTemplateInfo {
            id: "3d-scene",
            title: "3D Scene",
            category: "Core",
            description: "Reserved for the 3D scene workflow.",
            enabled: false,
        },
        ProjectTemplateInfo {
            id: "sample-world",
            title: "Sample World",
            category: "Sample",
            description: "Reserved for sample content generation.",
            enabled: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_renderable_empty_is_enabled_for_creation() {
        assert_eq!(
            ProjectTemplate::from_enabled_id("renderable-empty"),
            Some(ProjectTemplate::RenderableEmpty)
        );
        assert_eq!(ProjectTemplate::from_enabled_id("3d-scene"), None);
        assert_eq!(
            project_template_catalog()
                .iter()
                .filter(|template| template.enabled)
                .map(|template| template.id)
                .collect::<Vec<_>>(),
            vec!["renderable-empty"]
        );
    }

    #[test]
    fn create_request_preserves_name_and_validates_launch_fields() {
        let request =
            CreateProjectRequest::new("My Game", "E:/Projects", ProjectTemplate::RenderableEmpty);

        assert_eq!(request.project_name, "My Game");
        assert_eq!(
            request.target_root(),
            PathBuf::from("E:/Projects").join("My Game")
        );
        assert_eq!(request.validate_launch_fields(), Ok(()));

        let padded_name = CreateProjectRequest::new("  My Game  ", "E:/Projects", request.template);
        assert!(matches!(
            padded_name.validate_launch_fields(),
            Err(CreateProjectRequestError::ProjectName {
                source: ProjectNameError::SurroundingWhitespace { .. }
            })
        ));

        let missing_name = CreateProjectRequest::new("   ", "E:/Projects", request.template);
        assert_eq!(
            missing_name.validate_launch_fields(),
            Err(CreateProjectRequestError::ProjectName {
                source: ProjectNameError::Empty
            })
        );
        let missing_location = CreateProjectRequest::new("Game", "", request.template);
        assert_eq!(
            missing_location.validate_launch_fields(),
            Err(CreateProjectRequestError::MissingLocation)
        );
    }

    #[test]
    fn create_request_rejects_unsafe_filename_components() {
        for name in ["..", "folder/Game", r"folder\Game", "NUL", "Game.", "Game "] {
            let request =
                CreateProjectRequest::new(name, "E:/Projects", ProjectTemplate::RenderableEmpty);
            assert!(
                request.validate_launch_fields().is_err(),
                "accepted {name:?}"
            );
        }
    }
}
