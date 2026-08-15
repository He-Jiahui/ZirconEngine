use super::*;

pub(crate) fn build_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    let welcome = session.welcome_pane_snapshot(false);
    if let Some(descriptor_id) = session.open_builtin_view.as_deref() {
        editor_manager.dismiss_welcome_page()?;
        editor_manager.open_view(
            crate::ui::workbench::view::ViewDescriptorId::new(descriptor_id),
            None,
        )?;
        let mut state = EditorState::welcome_with_context(
            viewport_size,
            welcome,
            editor_manager.context().clone(),
        );
        state.set_session_mode(EditorSessionMode::Project);
        state.set_status_line(session.status_message.clone());
        return Ok(state);
    }

    match (session.mode, session.project.clone()) {
        (EditorSessionMode::Project | EditorSessionMode::Playing, Some(document)) => {
            editor_manager.apply_project_workspace(document.editor_workspace.clone())?;
            let authoring_world = editor_manager.prepare_authoring_world(document.world)?;
            let mut state = EditorState::project_with_context(
                authoring_world,
                viewport_size,
                document.root_path.to_string_lossy().into_owned(),
                editor_manager.context().clone(),
            );
            state.set_welcome_snapshot(welcome);
            state.set_status_line(session.status_message.clone());
            Ok(state)
        }
        (EditorSessionMode::Welcome | EditorSessionMode::Playing, _) => {
            let mut state = EditorState::welcome_with_context(
                viewport_size,
                welcome,
                editor_manager.context().clone(),
            );
            state.set_status_line(session.status_message.clone());
            Ok(state)
        }
        (EditorSessionMode::Project, None) => Err("startup session is missing project document"
            .to_string()
            .into()),
    }
}
