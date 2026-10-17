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
    pub template: ProjectTemplateId,
}

impl CreateProjectRequest {
    pub fn new(
        project_name: impl Into<String>,
        location: impl Into<PathBuf>,
        template: ProjectTemplateId,
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

pub fn enabled_project_template_id(id: &str) -> Option<ProjectTemplateId> {
    let id = ProjectTemplateId::parse(id)?;
    project_template_catalog()
        .iter()
        .any(|candidate| candidate.enabled && candidate.id == id.as_str())
        .then_some(id)
}

#[cfg(test)]
#[path = "tests/create_project_request.rs"]
mod tests;
