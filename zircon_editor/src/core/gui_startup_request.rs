use std::path::PathBuf;

use crate::ui::workbench::startup::{NewProjectDraft, NewProjectTemplate};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorGuiStartupRequest {
    OpenProject { project_path: PathBuf },
    OpenBuiltinView { descriptor_id: String },
    CreateProject(NewProjectDraft),
}

impl EditorGuiStartupRequest {
    pub fn open_project(project_path: impl Into<PathBuf>) -> Self {
        Self::OpenProject {
            project_path: project_path.into(),
        }
    }

    pub fn open_builtin_view(descriptor_id: impl Into<String>) -> Self {
        Self::OpenBuiltinView {
            descriptor_id: descriptor_id.into(),
        }
    }

    pub fn create_renderable_empty(
        project_name: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self::CreateProject(NewProjectDraft {
            project_name: project_name.into(),
            location: location.into(),
            template: NewProjectTemplate::RenderableEmpty,
        })
    }
}
