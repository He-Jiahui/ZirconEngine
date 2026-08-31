use crate::core::project::{NewProjectDraft, RecentProjectEntry};
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    /// Builds a chooser-only session; selecting a recent project remains an explicit action.
    pub(in crate::ui::host) fn resolve_startup_session(
        &self,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        zircon_runtime::profile_scope!("editor", "startup_session", "resolve_startup_session");

        let recent_projects = self.recent_projects_snapshot()?;
        Ok(project_chooser_startup_session(recent_projects))
    }
}

fn project_chooser_startup_session(
    recent_projects: Vec<RecentProjectEntry>,
) -> EditorStartupSessionDocument {
    EditorStartupSessionDocument {
        mode: EditorSessionMode::Welcome,
        project: None,
        open_builtin_view: None,
        recent_projects,
        draft: {
            zircon_runtime::profile_scope!("editor", "startup_session", "build_default_draft");
            NewProjectDraft::renderable_empty_default()
        },
        creation_validation: "Checking project location…".to_string(),
        can_open_existing: false,
        status_message: "Select a project to open.".to_string(),
    }
}
