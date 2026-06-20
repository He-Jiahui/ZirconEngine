use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn ensure_welcome_surface_bridge(&mut self) -> bool {
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

    pub(in crate::ui::retained_host::app) fn dispatch_welcome_surface_control(
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
