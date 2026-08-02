use std::error::Error;
use std::sync::Arc;

use super::super::super::super::*;
#[cfg(test)]
use super::super::session::resolve_startup_state;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::ui::host::EditorHostStartupSession;
#[cfg(test)]
use crate::ui::host::resolve_editor_startup_session;
use zircon_runtime::asset::project::ProjectManager;

pub(super) type StartupSessionState = EditorHostStartupSession;

pub(super) fn resolve_startup_session_state(
    editor_manager: Arc<EditorManager>,
    startup_request: Option<EditorGuiStartupRequest>,
    prepared_project: Option<ProjectManager>,
    viewport_size: UVec2,
) -> Result<StartupSessionState, Box<dyn Error>> {
    #[cfg(not(test))]
    {
        return EditorHostStartupSession::open_with_prepared_project(
            editor_manager,
            startup_request,
            prepared_project,
            viewport_size,
        );
    }

    #[cfg(test)]
    {
        let startup_session = match prepared_project {
            Some(project) => editor_manager.open_prepared_project_and_remember(project)?,
            None => resolve_editor_startup_session(editor_manager.as_ref(), startup_request)?,
        };
        let state =
            resolve_startup_state(editor_manager.as_ref(), &startup_session, viewport_size)?;
        Ok(EditorHostStartupSession::from_parts(
            startup_session,
            state,
            editor_manager,
        ))
    }
}
