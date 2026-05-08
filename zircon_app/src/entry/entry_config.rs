use super::entry_profile::EntryProfile;
use zircon_runtime::{
    core::framework::render::RenderProfileBundle, plugin::ExportProfile,
    plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeProfileDescriptor, plugin::RuntimeProfileId, RuntimePluginId, RuntimeTargetMode,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryConfig {
    pub profile: EntryProfile,
    pub runtime_profile: Option<RuntimeProfileId>,
    pub target_mode: RuntimeTargetMode,
    pub project_plugins: Option<ProjectPluginManifest>,
    pub export_profile: Option<ExportProfile>,
    pub render_profile: RenderProfileBundle,
    pub editor_enabled_subsystems: Option<Vec<String>>,
    pub editor_runtime_sandbox_enabled: bool,
}

impl EntryConfig {
    pub fn new(profile: EntryProfile) -> Self {
        Self {
            profile,
            runtime_profile: None,
            target_mode: default_target_mode_for_profile(profile),
            project_plugins: None,
            export_profile: None,
            render_profile: default_render_profile_for_profile(profile),
            editor_enabled_subsystems: None,
            editor_runtime_sandbox_enabled: true,
        }
    }

    pub fn for_runtime_profile(profile_id: RuntimeProfileId) -> Self {
        Self::new(entry_profile_for_runtime_profile(profile_id)).with_runtime_profile(profile_id)
    }

    pub fn with_runtime_profile(mut self, profile_id: RuntimeProfileId) -> Self {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        self.runtime_profile = Some(profile_id);
        self.target_mode = profile.target_mode;
        self.project_plugins = Some(profile.project_manifest());
        self
    }

    pub fn with_target_mode(mut self, target_mode: RuntimeTargetMode) -> Self {
        self.runtime_profile = None;
        self.target_mode = target_mode;
        self
    }

    pub fn with_required_runtime_plugins(mut self, plugins: impl AsRef<[RuntimePluginId]>) -> Self {
        self.runtime_profile = None;
        let mut selections = plugins
            .as_ref()
            .iter()
            .copied()
            .map(|id| ProjectPluginSelection::runtime_plugin(id, true, true))
            .collect::<Vec<_>>();
        selections.extend(self.optional_runtime_plugins());
        self.project_plugins = Some(ProjectPluginManifest { selections });
        self
    }

    pub fn with_optional_runtime_plugins(mut self, plugins: impl AsRef<[RuntimePluginId]>) -> Self {
        let mut selections = self
            .project_plugins
            .take()
            .map(|plugins| plugins.selections)
            .unwrap_or_default();
        selections.extend(
            plugins
                .as_ref()
                .iter()
                .copied()
                .map(|id| ProjectPluginSelection::runtime_plugin(id, true, false)),
        );
        self.project_plugins = Some(ProjectPluginManifest { selections });
        self
    }

    pub fn with_runtime_plugins(
        mut self,
        required: impl AsRef<[RuntimePluginId]>,
        optional: impl AsRef<[RuntimePluginId]>,
    ) -> Self {
        self.runtime_profile = None;
        let mut selections = required
            .as_ref()
            .iter()
            .copied()
            .map(|id| ProjectPluginSelection::runtime_plugin(id, true, true))
            .collect::<Vec<_>>();
        selections.extend(
            optional
                .as_ref()
                .iter()
                .copied()
                .map(|id| ProjectPluginSelection::runtime_plugin(id, true, false)),
        );
        self.project_plugins = Some(ProjectPluginManifest { selections });
        self
    }

    pub fn with_project_plugins(mut self, plugins: ProjectPluginManifest) -> Self {
        self.runtime_profile = None;
        self.project_plugins = Some(plugins);
        self
    }

    pub fn with_export_profile(mut self, export_profile: ExportProfile) -> Self {
        self.runtime_profile = None;
        self.target_mode = export_profile.target_mode;
        self.export_profile = Some(export_profile);
        self
    }

    pub fn with_render_profile(mut self, render_profile: RenderProfileBundle) -> Self {
        self.render_profile = render_profile;
        self
    }

    pub fn with_editor_enabled_subsystems<I, S>(mut self, subsystem_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.editor_enabled_subsystems = Some(subsystem_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_editor_runtime_sandbox_enabled(mut self, enabled: bool) -> Self {
        self.editor_runtime_sandbox_enabled = enabled;
        self
    }

    pub fn project_plugin_manifest(&self) -> Option<ProjectPluginManifest> {
        self.project_plugins.clone()
    }

    pub fn runtime_profile(&self) -> Option<RuntimeProfileId> {
        self.runtime_profile
    }

    fn optional_runtime_plugins(&self) -> Vec<ProjectPluginSelection> {
        self.project_plugins
            .as_ref()
            .map(|plugins| {
                plugins
                    .selections
                    .iter()
                    .filter(|selection| !selection.required)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

const fn entry_profile_for_runtime_profile(profile_id: RuntimeProfileId) -> EntryProfile {
    match profile_id {
        RuntimeProfileId::Editor | RuntimeProfileId::Dev => EntryProfile::Editor,
        RuntimeProfileId::Server => EntryProfile::Headless,
        RuntimeProfileId::Minimal | RuntimeProfileId::Client2d | RuntimeProfileId::Client3d => {
            EntryProfile::Runtime
        }
    }
}

const fn default_target_mode_for_profile(profile: EntryProfile) -> RuntimeTargetMode {
    match profile {
        EntryProfile::Editor => RuntimeTargetMode::EditorHost,
        EntryProfile::Runtime => RuntimeTargetMode::ClientRuntime,
        EntryProfile::Headless => RuntimeTargetMode::ServerRuntime,
    }
}

fn default_render_profile_for_profile(profile: EntryProfile) -> RenderProfileBundle {
    match profile {
        EntryProfile::Editor | EntryProfile::Runtime => RenderProfileBundle::default_render(),
        EntryProfile::Headless => RenderProfileBundle::headless(),
    }
}
