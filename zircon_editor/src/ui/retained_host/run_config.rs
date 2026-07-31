use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::plugin::EditorPluginRegistrationReport;
use std::path::PathBuf;
use zircon_runtime::asset::project::ProjectManager;

#[derive(Clone, Debug, Default)]
pub struct EditorHostRunConfig {
    startup_request: Option<EditorGuiStartupRequest>,
    startup_project: Option<ProjectManager>,
    exit_after_first_presented_frame: bool,
    first_presented_frame_capture_path: Option<PathBuf>,
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

    /// Transfers a project generation prepared by the application entry into the host.
    pub fn with_prepared_project(mut self, project: Option<ProjectManager>) -> Self {
        self.startup_project = project;
        self
    }

    pub fn with_exit_after_first_presented_frame(mut self, exit: bool) -> Self {
        self.exit_after_first_presented_frame = exit;
        self
    }

    /// Captures the retained host presentation after its first successful native present.
    pub fn with_first_presented_frame_capture_path(mut self, path: PathBuf) -> Self {
        self.first_presented_frame_capture_path = Some(path);
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

    pub fn first_presented_frame_capture_path(&self) -> Option<&std::path::Path> {
        self.first_presented_frame_capture_path.as_deref()
    }

    pub fn editor_plugin_registration_count(&self) -> usize {
        self.editor_plugin_registrations.len()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<EditorGuiStartupRequest>,
        Option<ProjectManager>,
        Option<PathBuf>,
        Vec<EditorPluginRegistrationReport>,
    ) {
        (
            self.startup_request,
            self.startup_project,
            self.first_presented_frame_capture_path,
            self.editor_plugin_registrations,
        )
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
    fn editor_host_run_config_carries_a_one_shot_presented_frame_capture_path() {
        let path = PathBuf::from("evidence/editor-first-frame.png");
        let config =
            EditorHostRunConfig::new().with_first_presented_frame_capture_path(path.clone());

        assert_eq!(
            config.first_presented_frame_capture_path(),
            Some(path.as_path())
        );
        let (_, _, capture_path, _) = config.into_parts();
        assert_eq!(capture_path, Some(path));
    }

    #[test]
    fn editor_host_run_config_carries_composition_root_plugin_registrations() {
        let descriptor = crate::core::plugin::EditorPluginDescriptor::new(
            "tests.composed",
            "Composed",
            "tests_composed_editor",
        );
        let registration = crate::core::plugin::EditorPluginRegistrationReport::from_plugin(
            &descriptor,
            descriptor.standalone_package_manifest(),
        );

        let config = EditorHostRunConfig::new().with_editor_plugin_registrations([registration]);

        assert_eq!(config.editor_plugin_registration_count(), 1);
        let (_, _, _, registrations) = config.into_parts();
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, "tests.composed");
    }

    #[test]
    fn prepared_project_startup_does_not_reopen_a_path() {
        let startup_source = include_str!("../host/editor_host_startup.rs");

        assert!(startup_source.contains("prepared_project"));
        assert!(startup_source.contains("open_prepared_project_and_remember"));
    }
}
