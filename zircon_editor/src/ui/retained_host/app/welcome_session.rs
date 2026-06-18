use super::*;

mod actions;

impl RetainedEditorHost {
    pub(super) fn ensure_welcome_surface_bridge(&mut self) -> bool {
        if self.welcome_surface_bridge.is_some() {
            return true;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "lazy_welcome_surface_bridge");
        match callback_dispatch::BuiltinWelcomeSurfaceTemplateBridge::new_minimal() {
            Ok(bridge) => {
                self.welcome_surface_bridge = Some(bridge);
                true
            }
            Err(error) => {
                self.set_status_line(format!("Failed to load welcome UI controls: {error}"));
                false
            }
        }
    }

    pub(super) fn refresh_welcome_snapshot(&mut self) {
        let snapshot = self.startup_session.welcome_pane_snapshot(false);
        self.runtime.set_welcome_snapshot(snapshot);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
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
            self.set_status_line(status_message);
            return Ok(());
        }

        match (mode, project) {
            (EditorSessionMode::Project | EditorSessionMode::Playing, Some(document)) => {
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
            (EditorSessionMode::Welcome | EditorSessionMode::Playing, _) => {
                self.runtime.set_session_mode(EditorSessionMode::Welcome);
                self.runtime.set_welcome_snapshot(welcome_snapshot);
                self.editor_manager
                    .show_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            }
            (EditorSessionMode::Project, None) => {
                return Err("startup session is missing project document".to_string());
            }
        }

        self.set_status_line(status_message);
        Ok(())
    }

    pub(super) fn dispatch_welcome_surface_control(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) {
        if !self.ensure_welcome_surface_bridge() {
            return;
        }
        let Some(welcome_surface_bridge) = self.welcome_surface_bridge.as_ref() else {
            self.set_status_line("Welcome UI controls are not available");
            return;
        };
        let Some(binding_control_id) = welcome_surface_binding_control_id(control_id) else {
            self.set_status_line(format!("Unknown welcome surface control {control_id}"));
            return;
        };
        let Some(result) = callback_dispatch::dispatch_builtin_welcome_surface_control(
            welcome_surface_bridge,
            binding_control_id,
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

fn welcome_surface_binding_control_id(action_or_control_id: &str) -> Option<&'static str> {
    match action_or_control_id {
        "ProjectNameEdited" | "welcome.project.name.edit" => Some("ProjectNameEdited"),
        "LocationEdited" | "welcome.project.location.edit" => Some("LocationEdited"),
        "CreateProject" | "welcome.project.create" => Some("CreateProject"),
        "OpenExistingProject" | "welcome.project.open_existing" => Some("OpenExistingProject"),
        _ => None,
    }
}
