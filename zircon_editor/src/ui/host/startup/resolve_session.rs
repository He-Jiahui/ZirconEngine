use crate::core::project::{NewProjectDraft, RecentProjectEntry, RecentProjectValidation};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use super::create_or_open::restored_project_status_message;

impl EditorUiHost {
    pub(in crate::ui::host) fn resolve_startup_session_with_project_open<OpenProject>(
        &self,
        open_project: OpenProject,
    ) -> Result<EditorStartupSessionDocument, EditorError>
    where
        OpenProject: FnOnce(&str) -> Result<EditorProjectDocument, EditorError>,
    {
        zircon_runtime::profile_scope!("editor", "startup_session", "resolve_startup_session");

        let recent_projects = self.recent_projects_snapshot()?;
        let Some(last_project) = recent_projects
            .iter()
            .find(|project| project.validation == RecentProjectValidation::Valid)
        else {
            return Ok(component_showcase_startup_session(
                recent_projects,
                "No valid recent project could be restored; Opened UI Component Showcase"
                    .to_string(),
            ));
        };

        let last_project_path = last_project.path.clone();
        let document = match open_project(&last_project_path) {
            Ok(document) => document,
            Err(error) => {
                return Ok(component_showcase_startup_session(
                    recent_projects,
                    format!(
                        "Could not restore recent project {}: {}; Opened UI Component Showcase",
                        last_project_path, error
                    ),
                ));
            }
        };

        let status_message = restored_project_status_message(&document);
        Ok(EditorStartupSessionDocument {
            mode: EditorSessionMode::Project,
            project: Some(document),
            open_builtin_view: None,
            recent_projects: self.recent_projects_snapshot()?,
            draft: NewProjectDraft::renderable_empty_default(),
            creation_validation: String::new(),
            can_open_existing: false,
            status_message,
        })
    }
}

fn component_showcase_startup_session(
    recent_projects: Vec<RecentProjectEntry>,
    status_message: String,
) -> EditorStartupSessionDocument {
    EditorStartupSessionDocument {
        mode: EditorSessionMode::Welcome,
        project: None,
        open_builtin_view: Some("editor.ui_component_showcase".to_string()),
        recent_projects,
        draft: {
            zircon_runtime::profile_scope!("editor", "startup_session", "build_default_draft");
            NewProjectDraft::renderable_empty_default()
        },
        creation_validation: "Checking project location…".to_string(),
        can_open_existing: false,
        status_message,
    }
}
