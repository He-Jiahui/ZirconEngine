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
#[path = "tests/run_config.rs"]
mod tests;
