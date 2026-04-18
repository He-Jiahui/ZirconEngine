use super::*;

pub(crate) fn build_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    let welcome = session.welcome_pane_snapshot(false);
    match session.mode {
        EditorSessionMode::Project => {
            let document = session
                .project
                .clone()
                .ok_or_else(|| "startup session is missing project document".to_string())?;
            editor_manager.apply_project_workspace(document.editor_workspace.clone())?;
            let level = editor_manager.create_runtime_level(document.world)?;
            let mut state = EditorState::project(
                level,
                viewport_size,
                document.root_path.to_string_lossy().into_owned(),
            );
            state.set_welcome_snapshot(welcome);
            state.set_status_line(session.status_message.clone());
            Ok(state)
        }
        EditorSessionMode::Welcome => {
            let mut state = EditorState::welcome(viewport_size, welcome);
            state.set_status_line(session.status_message.clone());
            Ok(state)
        }
    }
}
