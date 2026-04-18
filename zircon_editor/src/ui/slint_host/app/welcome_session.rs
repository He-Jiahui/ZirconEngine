use super::*;

impl SlintEditorHost {
    pub(super) fn refresh_welcome_snapshot(&mut self) {
        let snapshot = self.startup_session.welcome_pane_snapshot(false);
        self.runtime.set_welcome_snapshot(snapshot);
        self.presentation_dirty = true;
    }

    pub(super) fn present_welcome_surface(
        &mut self,
        status_message: impl Into<String>,
    ) -> Result<(), String> {
        self.startup_session.recent_projects = self
            .editor_manager
            .recent_projects_snapshot()
            .map_err(|error| error.to_string())?;
        self.startup_session.status_message = status_message.into();
        self.editor_manager
            .show_welcome_page()
            .map_err(|error| error.to_string())?;
        if !self.runtime.editor_snapshot().project_open {
            self.runtime.set_session_mode(EditorSessionMode::Welcome);
        }
        self.refresh_welcome_snapshot();
        Ok(())
    }

    pub(super) fn apply_startup_session(
        &mut self,
        mut session: EditorStartupSessionDocument,
    ) -> Result<(), String> {
        let welcome_snapshot = session.welcome_pane_snapshot(false);
        let status_message = session.status_message.clone();
        let mode = session.mode;
        let project = session.project.take();
        self.startup_session = session;

        match (mode, project) {
            (EditorSessionMode::Project, Some(document)) => {
                self.editor_manager
                    .apply_project_workspace(document.editor_workspace.clone())
                    .map_err(|error| error.to_string())?;
                let level = self
                    .editor_manager
                    .create_runtime_level(document.world)
                    .map_err(|error| error.to_string())?;
                self.runtime
                    .replace_world(level, document.root_path.to_string_lossy().into_owned());
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.runtime.set_welcome_snapshot(welcome_snapshot);
                self.editor_manager
                    .dismiss_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.sync_asset_workspace();
                self.mark_render_and_presentation_dirty();
            }
            (EditorSessionMode::Welcome, _) => {
                self.runtime.set_session_mode(EditorSessionMode::Welcome);
                self.runtime.set_welcome_snapshot(welcome_snapshot);
                self.editor_manager
                    .show_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.presentation_dirty = true;
            }
            (EditorSessionMode::Project, None) => {
                return Err("startup session is missing project document".to_string());
            }
        }

        self.set_status_line(status_message);
        Ok(())
    }

    pub(super) fn update_welcome_project_name(&mut self, value: &str) {
        self.startup_session.draft.project_name = value.to_string();
        self.refresh_welcome_snapshot();
    }

    pub(super) fn update_welcome_location(&mut self, value: &str) {
        self.startup_session.draft.location = value.to_string();
        self.refresh_welcome_snapshot();
    }

    pub(super) fn create_project_from_welcome(&mut self) {
        match self
            .editor_manager
            .create_project_and_open(self.startup_session.draft.clone())
            .map_err(|error| error.to_string())
            .and_then(|session| self.apply_startup_session(session))
        {
            Ok(()) => {}
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }

    pub(super) fn open_existing_project_from_welcome(&mut self) {
        let result = self
            .startup_session
            .draft
            .validate_for_open_existing()
            .map_err(|error| error.to_string())
            .and_then(|root| {
                self.editor_manager
                    .open_project_and_remember(root)
                    .map_err(|error| error.to_string())
            })
            .and_then(|session| self.apply_startup_session(session));
        if let Err(error) = result {
            self.startup_session.status_message = error.clone();
            self.refresh_welcome_snapshot();
            self.set_status_line(error);
        }
    }

    pub(super) fn open_recent_project(&mut self, path: &str) {
        let result = self
            .editor_manager
            .open_project_and_remember(path)
            .map_err(|error| error.to_string())
            .and_then(|session| self.apply_startup_session(session));
        if let Err(error) = result {
            self.startup_session.status_message = error.clone();
            if let Ok(recent_projects) = self.editor_manager.recent_projects_snapshot() {
                self.startup_session.recent_projects = recent_projects;
            }
            self.refresh_welcome_snapshot();
            self.set_status_line(error);
        }
    }

    pub(super) fn remove_recent_project(&mut self, path: &str) {
        match self
            .editor_manager
            .forget_recent_project(path)
            .map_err(|error| error.to_string())
            .and_then(|_| {
                self.editor_manager
                    .recent_projects_snapshot()
                    .map_err(|error| error.to_string())
            }) {
            Ok(recent_projects) => {
                self.startup_session.recent_projects = recent_projects;
                self.startup_session.status_message = format!("Removed recent project {path}");
                self.refresh_welcome_snapshot();
                self.set_status_line(format!("Removed recent project {path}"));
            }
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }

    pub(super) fn handle_welcome_surface_event(&mut self, event: WelcomeHostEvent) {
        match event {
            WelcomeHostEvent::SetProjectName { value } => {
                self.update_welcome_project_name(value.as_str());
            }
            WelcomeHostEvent::SetLocation { value } => {
                self.update_welcome_location(value.as_str());
            }
            WelcomeHostEvent::CreateProject => self.create_project_from_welcome(),
            WelcomeHostEvent::OpenExistingProject => self.open_existing_project_from_welcome(),
            WelcomeHostEvent::OpenRecentProject { path } => {
                self.open_recent_project(path.as_str());
            }
            WelcomeHostEvent::RemoveRecentProject { path } => {
                self.remove_recent_project(path.as_str());
            }
        }
    }

    pub(super) fn dispatch_welcome_surface_control(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) {
        let Some(result) = callback_dispatch::dispatch_builtin_welcome_surface_control(
            &self.welcome_surface_bridge,
            control_id,
            event_kind,
            arguments,
        ) else {
            self.set_status_line(format!("Unknown welcome surface control {control_id}"));
            return;
        };

        match result {
            Ok(event) => self.handle_welcome_surface_event(event),
            Err(error) => self.set_status_line(error),
        }
    }
}
