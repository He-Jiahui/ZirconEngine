use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::hub_link::HubEditorHandshake;
use crate::core::play::SharedPlayBackend;
use crate::core::plugin::EditorPluginRegistrationReport;
use zircon_runtime::asset::{project::ResolvedProjectPath, AssetUri};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

#[derive(Clone, Default)]
pub struct EditorHostRunConfig {
    startup_request: Option<EditorGuiStartupRequest>,
    project_runtime_build_set: Option<ZrRuntimeBuildSetId>,
    startup_scene_uri: Option<AssetUri>,
    startup_layout_preset: Option<String>,
    exit_after_first_presented_frame: bool,
    first_presented_frame_capture_path: Option<ResolvedProjectPath>,
    editor_plugin_registrations: Vec<EditorPluginRegistrationReport>,
    hub_handshake: Option<HubEditorHandshake>,
    play_backend: Option<SharedPlayBackend>,
}

impl EditorHostRunConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_startup_request(mut self, request: Option<EditorGuiStartupRequest>) -> Self {
        self.startup_request = request;
        self
    }

    /// Transfers the App-validated runtime BuildSet into Editor admission.
    pub fn with_project_runtime_build_set(mut self, build_set_id: ZrRuntimeBuildSetId) -> Self {
        self.project_runtime_build_set = Some(build_set_id);
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

    /// Installs the App-composed backend that owns Play runtime session creation and retirement.
    pub fn with_play_backend(mut self, backend: SharedPlayBackend) -> Self {
        self.play_backend = Some(backend);
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

    pub fn project_runtime_build_set(&self) -> Option<&ZrRuntimeBuildSetId> {
        self.project_runtime_build_set.as_ref()
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

    pub(crate) fn play_backend(&self) -> Option<SharedPlayBackend> {
        self.play_backend.clone()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<EditorGuiStartupRequest>,
        Option<ResolvedProjectPath>,
        Vec<EditorPluginRegistrationReport>,
        Option<ZrRuntimeBuildSetId>,
        Option<HubEditorHandshake>,
    ) {
        (
            self.startup_request,
            self.first_presented_frame_capture_path,
            self.editor_plugin_registrations,
            self.project_runtime_build_set,
            self.hub_handshake,
        )
    }
}

impl std::fmt::Debug for EditorHostRunConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EditorHostRunConfig")
            .field("startup_request", &self.startup_request)
            .field("project_runtime_build_set", &self.project_runtime_build_set)
            .field("startup_scene_uri", &self.startup_scene_uri)
            .field("startup_layout_preset", &self.startup_layout_preset)
            .field(
                "exit_after_first_presented_frame",
                &self.exit_after_first_presented_frame,
            )
            .field(
                "first_presented_frame_capture_path",
                &self.first_presented_frame_capture_path,
            )
            .field(
                "editor_plugin_registration_count",
                &self.editor_plugin_registrations.len(),
            )
            .field("hub_handshake", &self.hub_handshake)
            .field("play_backend_configured", &self.play_backend.is_some())
            .finish()
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
        let (_, capture_path, _, _, _) = config.into_parts();
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
        let (_, _, registrations, _, _) = config.into_parts();
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
    fn editor_host_run_config_carries_the_preflighted_runtime_build_set() {
        let build_set = zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId::parse(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("fixture BuildSet id must be valid");
        let config = EditorHostRunConfig::new().with_project_runtime_build_set(build_set.clone());

        assert_eq!(config.project_runtime_build_set(), Some(&build_set));
    }

    #[test]
    fn editor_host_run_config_carries_the_app_owned_play_backend() {
        let config = EditorHostRunConfig::new()
            .with_play_backend(std::sync::Arc::new(crate::core::play::NoopPlayBackend));

        assert!(config.play_backend().is_some());
    }

    #[test]
    fn host_config_cannot_bypass_project_admission() {
        let config_source = include_str!("run_config.rs");
        let product_config_source = config_source
            .split("#[cfg(test)]")
            .next()
            .expect("production config source must precede its tests");
        let startup_source = include_str!("../host/editor_host_startup.rs");
        let host_construction_source = include_str!("app/host_lifecycle/startup/with_viewport.rs");

        assert!(!product_config_source.contains("with_prepared_project"));
        assert!(!startup_source.contains("open_with_prepared_project"));
        assert!(startup_source.contains("execute_project_launch_intent"));
        assert!(host_construction_source.contains("configure_project_runtime_build_set"));
    }

    #[test]
    fn project_intents_require_a_preflighted_build_set_before_host_construction() {
        let host_source = include_str!("app.rs");
        let automation_source = include_str!("app/automation.rs");
        let build_set_gate = host_source
            .find("project startup requires an App-preflighted runtime BuildSet")
            .expect("project startup must reject a missing BuildSet");
        let host_construction = host_source
            .find("RetainedEditorHost::new(")
            .expect("retained host construction must remain explicit");

        assert!(build_set_gate < host_construction);
        assert!(
            automation_source
                .contains("project startup requires an App-preflighted runtime BuildSet"),
            "non-windowed project automation must reject a missing BuildSet too"
        );
    }
}
