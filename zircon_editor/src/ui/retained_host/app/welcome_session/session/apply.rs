use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn apply_startup_session(
        &mut self,
        mut session: EditorStartupSessionDocument,
    ) -> Result<(), String> {
        self.clear_welcome_project_probe();
        let welcome_snapshot = session.welcome_pane_snapshot(false);
        let status_message = session.status_message.clone();
        let startup_view = session.open_builtin_view.clone();
        let mode = session.mode;
        let project = session.project.take();
        self.startup_session = session;

        if let Some(descriptor_id) = startup_view {
            self.editor_manager
                .dismiss_welcome_page()
                .map_err(|error| error.to_string())?;
            self.editor_manager
                .open_view(
                    crate::ui::workbench::view::ViewDescriptorId::new(descriptor_id),
                    None,
                )
                .map_err(|error| error.to_string())?;
            self.runtime.set_session_mode(EditorSessionMode::Project);
            self.runtime.set_welcome_snapshot(welcome_snapshot);
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            self.sync_hub_focus_binding()?;
            self.set_status_line(status_message);
            return Ok(());
        }

        match (mode, project) {
            (EditorSessionMode::Project | EditorSessionMode::Playing, Some(document)) => {
                self.editor_manager
                    .apply_project_workspace(document.editor_workspace.clone())
                    .map_err(|error| error.to_string())?;
                let project_root = document.root_path.clone();
                let default_scene = document.manifest.default_scene.clone();
                let authoring_world = self
                    .editor_manager
                    .prepare_authoring_world(document.world)
                    .map_err(|error| error.to_string())?;
                self.runtime
                    .replace_world(authoring_world, project_root.to_string_lossy().into_owned())
                    .map_err(|error| error.to_string())?;
                let scene_document = self
                    .editor_manager
                    .activate_startup_scene_document(&project_root, &default_scene)
                    .map_err(|error| error.to_string())?;
                self.runtime.bind_scene_document(scene_document);
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.runtime.set_welcome_snapshot(welcome_snapshot);
                self.editor_manager
                    .dismiss_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.sync_asset_workspace();
                self.mark_render_and_presentation_dirty();
            }
            (EditorSessionMode::Welcome | EditorSessionMode::Playing, _) => {
                self.runtime
                    .clear_project(welcome_snapshot)
                    .map_err(|error| error.to_string())?;
                self.editor_manager
                    .show_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            }
            (EditorSessionMode::Project, None) => {
                return Err("startup session is missing project document".to_string());
            }
        }

        self.sync_hub_focus_binding()?;
        self.set_status_line(status_message);
        Ok(())
    }
}
