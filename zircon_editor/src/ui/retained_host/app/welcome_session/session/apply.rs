use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn apply_startup_session(
        &mut self,
        session: EditorStartupSessionDocument,
    ) -> Result<(), String> {
        let previous_startup_session = self.startup_session.clone();
        let previous_workspace = self.editor_manager.project_workspace();
        let previous_runtime_snapshot = self.runtime.editor_snapshot();
        let previous_scene = self
            .runtime
            .project_scene_snapshot()
            .map_err(|error| error.to_string())?;
        let previous_viewport_sessions = self.runtime.scene_viewport_workspace_sessions();
        let previous_focused_view = self.editor_manager.current_focused_view();
        let previous_scene_identity = self.editor_manager.active_scene_identity_for_session();
        let should_abort_admitted_session =
            !previous_runtime_snapshot.project_open && previous_scene_identity.is_none();

        let result = self.apply_startup_session_transaction(session);
        if result.is_err() {
            let rollback = self.rollback_startup_session(
                previous_startup_session,
                previous_workspace,
                previous_runtime_snapshot,
                previous_scene,
                previous_viewport_sessions,
                previous_focused_view,
                previous_scene_identity,
            );
            if let Err(rollback_error) = rollback {
                return Err(format!(
                    "startup session failed and rollback failed: {}; {rollback_error}",
                    result.as_ref().unwrap_err()
                ));
            }
            // Project admission commits the manager-owned Ready guard before this retained-host
            // projection can activate its world, scene journal, focus, or presentation. If this
            // was a welcome-to-project transition, close that admitted session through the same
            // durable manager close phases used by the normal UI close path; clearing only the
            // runtime would leave the Ready guard and project owner hidden from the next launch.
            if should_abort_admitted_session
                && self
                    .editor_manager
                    .active_project_session_focus_target()
                    .is_some()
            {
                if let Err(close_error) = self.abort_admitted_project_session() {
                    return Err(format!(
                        "startup session failed; retained rollback succeeded but admitted project close failed: {}; {close_error}",
                        result.as_ref().unwrap_err()
                    ));
                }
                if let Err(focus_error) = self.sync_hub_focus_binding() {
                    return Err(format!(
                        "startup session failed; admitted project closed but Hub focus rollback failed: {}; {focus_error}",
                        result.as_ref().unwrap_err()
                    ));
                }
            }
        }
        result
    }

    fn abort_admitted_project_session(&mut self) -> Result<(), String> {
        self.commit_project_close()
            .map_err(|error| error.to_string())
    }

    fn apply_startup_session_transaction(
        &mut self,
        mut session: EditorStartupSessionDocument,
    ) -> Result<(), String> {
        self.clear_welcome_project_probe();
        let welcome_snapshot = session.welcome_pane_snapshot(false);
        let status_message = session.status_message.clone();
        let startup_view = session.open_builtin_view.clone();
        let mode = session.mode;
        // Consume the prepared project world only after the rollback snapshot is captured. The
        // committed startup session keeps the existing single-owner policy and does not retain a
        // second world clone after the activation transaction succeeds.
        let project = session.project.take();

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
            self.runtime.ensure_live_scene_viewport_sessions();
            self.runtime.set_session_mode(EditorSessionMode::Project);
            self.runtime.set_welcome_snapshot(welcome_snapshot);
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            self.sync_hub_focus_binding()?;
            self.set_status_line(status_message);
            self.startup_session = session;
            return Ok(());
        }

        match (mode, project) {
            (EditorSessionMode::Project | EditorSessionMode::Playing, Some(document)) => {
                let viewport_sessions = document
                    .editor_workspace
                    .as_ref()
                    .map(|workspace| {
                        workspace
                            .scene_viewport_sessions
                            .iter()
                            .map(|(view_id, snapshot)| {
                                (
                                    crate::core::editor_event::ViewInstanceId::new(
                                        view_id.0.clone(),
                                    ),
                                    snapshot.clone(),
                                )
                            })
                            .collect::<std::collections::BTreeMap<_, _>>()
                    })
                    .unwrap_or_default();
                let focused_view = document
                    .editor_workspace
                    .as_ref()
                    .and_then(|workspace| workspace.focused_view.clone());
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
                let focused_core_view = focused_view
                    .map(|view_id| crate::core::editor_event::ViewInstanceId::new(view_id.0));
                self.runtime.restore_scene_viewport_workspace_sessions(
                    &viewport_sessions,
                    focused_core_view.as_ref(),
                );
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
        self.startup_session = session;
        Ok(())
    }

    fn rollback_startup_session(
        &mut self,
        previous_startup_session: EditorStartupSessionDocument,
        previous_workspace: crate::ui::workbench::project::ProjectEditorWorkspace,
        previous_runtime_snapshot: crate::ui::workbench::snapshot::EditorDataSnapshot,
        previous_scene: Option<zircon_runtime::scene::Scene>,
        previous_viewport_sessions: std::collections::BTreeMap<
            crate::core::editor_event::ViewInstanceId,
            crate::scene::viewport::SceneViewportWorkspaceSessionSnapshot,
        >,
        previous_focused_view: Option<crate::ui::workbench::view::ViewInstanceId>,
        previous_scene_identity: Option<crate::core::document::ActiveSceneDocumentIdentity>,
    ) -> Result<(), String> {
        let mut errors = Vec::new();
        let active_scene_before_rollback = self.editor_manager.active_scene_identity_for_session();

        if let Err(error) = self
            .editor_manager
            .apply_project_workspace(Some(previous_workspace))
        {
            errors.push(format!("workspace: {error}"));
        }

        let previous_project_path = previous_runtime_snapshot.project_path.clone();
        match previous_scene {
            Some(scene) => match self.editor_manager.prepare_authoring_world(scene) {
                Ok(authoring_world) => {
                    if let Err(error) = self
                        .runtime
                        .replace_world(authoring_world, previous_project_path)
                    {
                        errors.push(format!("world: {error}"));
                    }
                }
                Err(error) => errors.push(format!("world preparation: {error}")),
            },
            None => {
                if let Err(error) = self
                    .runtime
                    .clear_project(previous_runtime_snapshot.welcome.clone())
                {
                    errors.push(format!("world: {error}"));
                }
            }
        }

        if let Some(identity) = previous_scene_identity.as_ref() {
            match zircon_runtime::asset::AssetUri::parse(identity.scene_uri()) {
                Ok(scene_uri) => {
                    match self
                        .editor_manager
                        .activate_startup_scene_document(identity.project_root(), &scene_uri)
                    {
                        Ok(document) => self.runtime.bind_scene_document(document),
                        Err(error) => errors.push(format!("scene journal: {error}")),
                    }
                }
                Err(error) => errors.push(format!("scene URI: {error}")),
            }
        } else {
            if let Some(identity) = active_scene_before_rollback {
                self.editor_manager
                    .clear_active_scene_document(identity.project_root());
            }
            self.runtime.clear_scene_document_binding();
        }

        if let Some(focused_view) = previous_focused_view.as_ref() {
            if let Err(error) = self.editor_manager.focus_view(focused_view) {
                errors.push(format!("focus: {error}"));
            }
        }
        let previous_focused_core = previous_focused_view
            .as_ref()
            .map(|view_id| crate::core::editor_event::ViewInstanceId::new(view_id.0.clone()));
        self.runtime.restore_scene_viewport_workspace_sessions(
            &previous_viewport_sessions,
            previous_focused_core.as_ref(),
        );
        self.runtime
            .set_session_mode(previous_runtime_snapshot.session_mode);
        self.runtime
            .set_welcome_snapshot(previous_runtime_snapshot.welcome);
        if previous_runtime_snapshot.session_mode == EditorSessionMode::Welcome {
            if let Err(error) = self.editor_manager.show_welcome_page() {
                errors.push(format!("welcome page: {error}"));
            }
        } else if let Err(error) = self.editor_manager.dismiss_welcome_page() {
            errors.push(format!("workbench page: {error}"));
        }
        self.startup_session = previous_startup_session;
        self.sync_asset_workspace();
        self.mark_render_and_presentation_dirty();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}
