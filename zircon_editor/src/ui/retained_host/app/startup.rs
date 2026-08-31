use super::*;

pub(crate) fn build_startup_state(
    editor_manager: &EditorManager,
    session: &mut EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    let welcome = session.welcome_pane_snapshot(false);
    let project = session.project.take();
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

    match (session.mode, project) {
        (EditorSessionMode::Project | EditorSessionMode::Playing, Some(document)) => {
            editor_manager.apply_project_workspace(document.editor_workspace)?;
            let project_root = document.root_path.clone();
            let default_scene = document.manifest.default_scene.clone();
            let authoring_world = editor_manager.prepare_authoring_world(document.world)?;
            let mut state = EditorState::project_with_context(
                authoring_world,
                viewport_size,
                project_root.to_string_lossy().into_owned(),
                editor_manager.context().clone(),
            );
            let scene_document =
                editor_manager.activate_startup_scene_document(&project_root, &default_scene)?;
            state.bind_scene_document(scene_document);
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
