use crate::core::project::{
    NewProjectDraft, ProjectAuthority, RecentProjectEntry, RecentProjectValidation,
};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use super::create_or_open::restored_project_status_message;

impl EditorUiHost {
    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        zircon_runtime::profile_scope!("editor", "startup_session", "resolve_startup_session");

        let stored = self.load_startup_session()?;
        let recent_projects =
            ProjectAuthority::default().recent_projects_with_validation(&stored, |path| {
                zircon_runtime::profile_scope!(
                    "editor",
                    "startup_session",
                    "validate_recent_project"
                );
                ProjectAuthority::default().validate_recent_project(path)
            });

        let Some(last_project_path) = stored
            .last_project_path
            .as_deref()
            .filter(|path| !path.trim().is_empty())
        else {
            return Ok(component_showcase_startup_session(
                recent_projects,
                "Opened UI Component Showcase".to_string(),
            ));
        };

        let last_project_validation =
            validation_for_recent_project(&recent_projects, last_project_path).unwrap_or_else(
                || ProjectAuthority::default().validate_recent_project(last_project_path),
            );
        if last_project_validation != RecentProjectValidation::Valid {
            return Ok(component_showcase_startup_session(
                recent_projects,
                format!(
                    "Could not restore last project {}: {}; Opened UI Component Showcase",
                    last_project_path,
                    recent_project_validation_message(last_project_validation)
                ),
            ));
        }

        let document = match self.open_project(last_project_path) {
            Ok(document) => document,
            Err(error) => {
                return Ok(component_showcase_startup_session(
                    recent_projects,
                    format!(
                        "Could not restore last project {}: {}; Opened UI Component Showcase",
                        last_project_path, error
                    ),
                ));
            }
        };

        // A restored legacy session can still point at zircon-project.toml. Record the opened
        // document root so future startup and removal operations use one project identity.
        self.remember_opened_project(&document.root_path, document.manifest.summary())?;
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

fn validation_for_recent_project(
    recent_projects: &[RecentProjectEntry],
    path: &str,
) -> Option<RecentProjectValidation> {
    recent_projects
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.validation)
}

fn recent_project_validation_message(validation: RecentProjectValidation) -> &'static str {
    match validation {
        RecentProjectValidation::Valid => "project is valid",
        RecentProjectValidation::Missing => "project is missing",
        RecentProjectValidation::InvalidManifest => "project manifest is invalid",
        RecentProjectValidation::InvalidProject => "project is invalid",
    }
}
