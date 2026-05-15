use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
        Self {
            project_name: project_name.into(),
            location: location.into(),
            template,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectTemplate {
    #[default]
    RenderableEmpty,
}

impl ProjectTemplate {
    pub fn as_editor_arg(self) -> &'static str {
        match self {
            Self::RenderableEmpty => "renderable-empty",
        }
    }
}
