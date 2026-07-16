use crate::core::editor_plugin::EditorPluginRegistrationReport;
use crate::core::gui_startup_request::EditorGuiStartupRequest;

#[derive(Clone, Debug, Default)]
pub struct EditorHostRunConfig {
    startup_request: Option<EditorGuiStartupRequest>,
    exit_after_first_presented_frame: bool,
    editor_plugin_registrations: Vec<EditorPluginRegistrationReport>,
}

impl EditorHostRunConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_startup_request(mut self, request: Option<EditorGuiStartupRequest>) -> Self {
        self.startup_request = request;
        self
    }

    pub fn with_exit_after_first_presented_frame(mut self, exit: bool) -> Self {
        self.exit_after_first_presented_frame = exit;
        self
    }

    pub fn with_editor_plugin_registrations(
        mut self,
        registrations: impl IntoIterator<Item = EditorPluginRegistrationReport>,
    ) -> Self {
        self.editor_plugin_registrations.extend(registrations);
        self
    }

    pub fn startup_request(&self) -> Option<&EditorGuiStartupRequest> {
        self.startup_request.as_ref()
    }

    pub fn exit_after_first_presented_frame(&self) -> bool {
        self.exit_after_first_presented_frame
    }

    pub fn editor_plugin_registration_count(&self) -> usize {
        self.editor_plugin_registrations.len()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<EditorGuiStartupRequest>,
        Vec<EditorPluginRegistrationReport>,
    ) {
        (self.startup_request, self.editor_plugin_registrations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_host_run_config_defaults_to_normal_interactive_startup() {
        let config = EditorHostRunConfig::new();

        assert_eq!(config.startup_request(), None);
        assert!(!config.exit_after_first_presented_frame());
    }

    #[test]
    fn editor_host_run_config_can_request_first_frame_exit() {
        let config = EditorHostRunConfig::new()
            .with_startup_request(Some(EditorGuiStartupRequest::open_builtin_view(
                "editor.material_component_lab",
            )))
            .with_exit_after_first_presented_frame(true);

        assert_eq!(
            config.startup_request(),
            Some(&EditorGuiStartupRequest::open_builtin_view(
                "editor.material_component_lab"
            ))
        );
        assert!(config.exit_after_first_presented_frame());
    }

    #[test]
    fn editor_host_run_config_carries_composition_root_plugin_registrations() {
        let descriptor = crate::core::editor_plugin::EditorPluginDescriptor::new(
            "tests.composed",
            "Composed",
            "tests_composed_editor",
        );
        let registration = crate::core::editor_plugin::EditorPluginRegistrationReport::from_plugin(
            &descriptor,
            descriptor.standalone_package_manifest(),
        );

        let config = EditorHostRunConfig::new().with_editor_plugin_registrations([registration]);

        assert_eq!(config.editor_plugin_registration_count(), 1);
        let (_, registrations) = config.into_parts();
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, "tests.composed");
    }
}
