use std::error::Error;

use super::super::super::*;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::project::NewProjectDraft;

pub(super) fn resolve_editor_startup_session(
    editor_manager: &EditorManager,
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<EditorStartupSessionDocument, crate::ui::host::EditorError> {
    match startup_request {
        Some(EditorGuiStartupRequest::OpenProject { project_path }) => {
            editor_manager.open_project_and_remember(project_path)
        }
        Some(EditorGuiStartupRequest::OpenBuiltinView { descriptor_id }) => {
            Ok(EditorStartupSessionDocument {
                mode: EditorSessionMode::Welcome,
                project: None,
                open_builtin_view: Some(descriptor_id.clone()),
                recent_projects: Vec::new(),
                draft: NewProjectDraft::renderable_empty_default(),
                status_message: format!("Opened {descriptor_id}"),
            })
        }
        Some(EditorGuiStartupRequest::CreateProject(draft)) => {
            editor_manager.create_project_and_open(draft)
        }
        None => editor_manager.resolve_startup_session(),
    }
}

#[cfg(not(test))]
pub(super) fn resolve_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    build_startup_state(editor_manager, session, viewport_size)
}

#[cfg(test)]
pub(super) fn resolve_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    build_startup_state(editor_manager, session, viewport_size).or_else(|error| {
        let message = error.to_string();
        if message.contains("SceneModule.Manager.DefaultLevelManager") {
            let mut state =
                EditorState::welcome(viewport_size, session.welcome_pane_snapshot(false));
            state.set_status_line(session.status_message.clone());
            Ok(state)
        } else {
            Err(error)
        }
    })
}
