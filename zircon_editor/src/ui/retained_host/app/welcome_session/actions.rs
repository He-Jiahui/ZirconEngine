use super::super::*;

impl RetainedEditorHost {
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

    fn open_startup_view(&mut self, descriptor_id: &str, status: &str) {
        match self
            .editor_manager
            .dismiss_welcome_page()
            .map_err(|error| error.to_string())
            .and_then(|_| {
                self.editor_manager
                    .open_view(
                        crate::ui::workbench::view::ViewDescriptorId::new(descriptor_id),
                        None,
                    )
                    .map_err(|error| error.to_string())
            }) {
            Ok(_) => {
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                self.set_status_line(status.to_string());
            }
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn handle_welcome_surface_event(
        &mut self,
        event: WelcomeHostEvent,
    ) {
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
            WelcomeHostEvent::OpenStartupWorkbench => {
                let _ = self.editor_manager.dismiss_welcome_page();
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                self.set_status_line("Opened default workbench".to_string());
            }
            WelcomeHostEvent::OpenStartupDemo => {
                self.open_startup_view(
                    "editor.ui_component_showcase",
                    "Opened UI Component Showcase",
                );
            }
            WelcomeHostEvent::OpenStartupAssetWindow => {
                self.open_startup_view(
                    "editor.asset_browser_window",
                    "Opened asset browser window",
                );
            }
            WelcomeHostEvent::OpenStartupUILayoutEditor => {
                self.open_startup_view(
                    "editor.ui_asset_editor_window",
                    "Opened UI layout editor window",
                );
            }
        }
    }
}
