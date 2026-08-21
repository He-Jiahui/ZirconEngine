use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::hub_link::HubEditorHandshake;
use crate::core::plugin::EditorPluginRegistrationReport;
use zircon_runtime::asset::{
    project::{ProjectManager, ResolvedProjectPath},
    AssetUri,
};

#[derive(Clone, Debug, Default)]
pub struct EditorHostRunConfig {
    startup_request: Option<EditorGuiStartupRequest>,
    startup_project: Option<ProjectManager>,
    startup_scene_uri: Option<AssetUri>,
    startup_layout_preset: Option<String>,
    exit_after_first_presented_frame: bool,
    first_presented_frame_capture_path: Option<ResolvedProjectPath>,
    editor_plugin_registrations: Vec<EditorPluginRegistrationReport>,
    hub_handshake: Option<HubEditorHandshake>,
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

    /// Opens this project-owned scene through the host document route after startup completes.
    pub fn with_startup_scene_uri(mut self, scene_uri: AssetUri) -> Self {
        self.startup_scene_uri = Some(scene_uri);
        self
    }

    /// Applies this existing layout preset after the host has opened its startup project.
    pub fn with_startup_layout_preset(mut self, preset: impl Into<String>) -> Self {
        self.startup_layout_preset = Some(preset.into());
        self
    }

    pub fn with_exit_after_first_presented_frame(mut self, exit: bool) -> Self {
        self.exit_after_first_presented_frame = exit;
        self
    }

    /// Captures the retained host presentation after its first successful native present.
    pub fn with_first_presented_frame_capture_path(mut self, path: ResolvedProjectPath) -> Self {
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

    /// Requests a terminal Hub mailbox outcome once the retained host reaches its startup gate.
    ///
    /// The application composition root uses this to transfer its verified Hub session into the
    /// editor host without exposing the retained-host handshake representation.
    pub fn with_hub_handshake(
        mut self,
        project_root: impl Into<std::path::PathBuf>,
        session: zircon_runtime_interface::hub_protocol::HubSessionToken,
    ) -> Self {
        self.hub_handshake = Some(HubEditorHandshake::new(project_root, session));
        self
    }

    pub fn startup_request(&self) -> Option<&EditorGuiStartupRequest> {
        self.startup_request.as_ref()
    }

    pub fn exit_after_first_presented_frame(&self) -> bool {
        self.exit_after_first_presented_frame
    }

    pub fn startup_layout_preset(&self) -> Option<&str> {
        self.startup_layout_preset.as_deref()
    }

    pub fn startup_scene_uri(&self) -> Option<&AssetUri> {
        self.startup_scene_uri.as_ref()
    }

    /// Returns both operation and display views so callers cannot accidentally log an
    /// operation-only Windows path.
    pub fn first_presented_frame_capture_path(&self) -> Option<&ResolvedProjectPath> {
        self.first_presented_frame_capture_path.as_ref()
    }

    pub fn editor_plugin_registration_count(&self) -> usize {
        self.editor_plugin_registrations.len()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<EditorGuiStartupRequest>,
        Option<ProjectManager>,
        Option<ResolvedProjectPath>,
        Vec<EditorPluginRegistrationReport>,
        Option<HubEditorHandshake>,
    ) {
        (
            self.startup_request,
            self.startup_project,
            self.first_presented_frame_capture_path,
            self.editor_plugin_registrations,
            self.hub_handshake,
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
        let (_, _, _, _, handshake) = config.into_parts();
        assert_eq!(handshake, None);
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
    fn editor_host_run_config_carries_a_startup_layout_preset() {
        let config = EditorHostRunConfig::new().with_startup_layout_preset("debug");

        assert_eq!(config.startup_layout_preset(), Some("debug"));
    }

    #[test]
    fn editor_host_run_config_carries_a_startup_scene_uri() {
        let scene_uri = zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml")
            .expect("scene URI should parse");
        let config = EditorHostRunConfig::new().with_startup_scene_uri(scene_uri.clone());

        assert_eq!(config.startup_scene_uri(), Some(&scene_uri));
    }

    #[test]
    fn startup_layout_loads_after_plugin_templates_and_before_first_refresh() {
        let host_source = include_str!("app.rs");
        let plugin_templates = host_source
            .find("retained_host.sync_plugin_template_documents_if_changed()?;")
            .expect("host startup should synchronize plugin templates");
        let load_preset = host_source
            .find(".apply_layout_command(LayoutCommand::LoadPreset { name })?")
            .expect("host startup should apply the requested layout preset");
        let first_refresh = host_source
            .find("host.borrow_mut().refresh_ui();")
            .expect("host startup should refresh the UI after initialization");

        assert!(plugin_templates < load_preset);
        assert!(load_preset < first_refresh);
    }

    #[test]
    fn startup_scene_loads_after_plugin_templates_and_before_layout_or_refresh() {
        let host_source = include_str!("app.rs");
        let plugin_templates = host_source
            .find("retained_host.sync_plugin_template_documents_if_changed()?;")
            .expect("host startup should synchronize plugin templates");
        let open_scene = host_source
            .find(".open_startup_scene(scene_uri)")
            .expect("host startup should submit the requested scene through the document route");
        let load_preset = host_source
            .find(".apply_layout_command(LayoutCommand::LoadPreset { name })?")
            .expect("host startup should apply the requested layout preset");
        let wire_callbacks = host_source
            .find("wire_callbacks(&ui, &host);")
            .expect("host startup should wire callbacks after retained host initialization");
        let first_refresh = host_source
            .find("host.borrow_mut().refresh_ui();")
            .expect("host startup should refresh the UI after initialization");

        assert!(plugin_templates < open_scene);
        assert!(open_scene < load_preset);
        assert!(open_scene < wire_callbacks);
        assert!(open_scene < first_refresh);
    }

    #[test]
    fn editor_host_run_config_carries_a_one_shot_presented_frame_capture_path() {
        let path = std::path::PathBuf::from("evidence/editor-first-frame.png");
        let resolved_path = zircon_runtime::asset::project::ProjectPaths::resolve_path(&path)
            .expect("capture path should resolve");
        let config = EditorHostRunConfig::new()
            .with_first_presented_frame_capture_path(resolved_path.clone());

        assert_eq!(
            config.first_presented_frame_capture_path(),
            Some(&resolved_path)
        );
        let (_, _, capture_path, _, _) = config.into_parts();
        assert_eq!(capture_path, Some(resolved_path));
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
        let (_, _, _, registrations, _) = config.into_parts();
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, "tests.composed");
    }

    #[test]
    fn editor_host_run_config_carries_a_hub_handshake() {
        use std::str::FromStr;

        let session = zircon_runtime_interface::hub_protocol::HubSessionToken::from_str(
            "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
        )
        .expect("valid Hub session");
        let config = EditorHostRunConfig::new().with_hub_handshake("E:/Projects/My Game", session);

        let (_, _, _, _, handshake) = config.into_parts();
        let handshake = handshake.expect("Hub handshake");
        assert_eq!(handshake.session(), session);
        assert_eq!(
            handshake.mailbox_path(),
            std::path::PathBuf::from("E:/Projects/My Game")
                .join(".zircon")
                .join("hub")
                .join(format!("{session}.json"))
        );
    }

    #[test]
    fn prepared_project_startup_does_not_reopen_a_path() {
        let startup_source = include_str!("../host/editor_host_startup.rs");

        assert!(startup_source.contains("prepared_project"));
        assert!(startup_source.contains("open_prepared_project_and_remember"));
    }
}
