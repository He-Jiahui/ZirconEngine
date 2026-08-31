use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ProjectPluginManifest, RuntimeProfileId,
};
use zircon_runtime::core::framework::render::RenderProfileBundle;
use zircon_runtime::core::framework::window::WindowDescriptor;

use crate::entry::EntryProfile;

use super::{EntryConfig, ProductConfigSource, ProductRoleRequest};

impl EntryConfig {
    pub fn new(profile: EntryProfile) -> Self {
        Self::for_product_role(ProductRoleRequest::from_entry_profile(profile))
    }

    pub fn for_product_role(role: ProductRoleRequest) -> Self {
        Self {
            role,
            runtime_profile: None,
            target_mode: None,
            project_plugins: None,
            project_plugins_source: None,
            export_profile: None,
            render_profile: None,
            window_descriptor: None,
            editor_enabled_subsystems: None,
            editor_runtime_sandbox_enabled: None,
            required_runtime_plugins: Vec::new(),
            optional_runtime_plugins: Vec::new(),
        }
    }

    pub fn for_runtime_profile(profile_id: RuntimeProfileId) -> Self {
        Self::for_product_role(ProductRoleRequest::from_runtime_profile(profile_id))
            .with_runtime_profile(profile_id)
    }

    pub fn with_runtime_profile(mut self, profile_id: RuntimeProfileId) -> Self {
        self.runtime_profile = Some(profile_id);
        self
    }

    pub fn with_target_mode(mut self, target_mode: RuntimeTargetMode) -> Self {
        self.target_mode = Some(target_mode);
        self
    }

    pub fn with_required_runtime_plugins(mut self, plugins: impl AsRef<[RuntimePluginId]>) -> Self {
        self.required_runtime_plugins = plugins.as_ref().to_vec();
        self
    }

    pub fn with_optional_runtime_plugins(mut self, plugins: impl AsRef<[RuntimePluginId]>) -> Self {
        self.optional_runtime_plugins
            .extend_from_slice(plugins.as_ref());
        self
    }

    pub fn with_runtime_plugins(
        mut self,
        required: impl AsRef<[RuntimePluginId]>,
        optional: impl AsRef<[RuntimePluginId]>,
    ) -> Self {
        self.required_runtime_plugins = required.as_ref().to_vec();
        self.optional_runtime_plugins = optional.as_ref().to_vec();
        self
    }

    pub fn with_project_plugins(mut self, plugins: ProjectPluginManifest) -> Self {
        self.project_plugins = Some(plugins);
        self.project_plugins_source = Some(ProductConfigSource::EntryRequest);
        self
    }

    pub(in crate::entry) fn with_export_project_plugins(
        mut self,
        plugins: ProjectPluginManifest,
    ) -> Self {
        self.project_plugins = Some(plugins);
        self.project_plugins_source = Some(ProductConfigSource::ExportProfile);
        self
    }

    pub fn with_export_profile(mut self, export_profile: ExportProfile) -> Self {
        self.export_profile = Some(export_profile);
        self
    }

    pub fn with_render_profile(mut self, render_profile: RenderProfileBundle) -> Self {
        self.render_profile = Some(render_profile);
        self
    }

    pub fn with_window_descriptor(mut self, window_descriptor: WindowDescriptor) -> Self {
        self.window_descriptor = Some(window_descriptor);
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
        self.editor_runtime_sandbox_enabled = Some(enabled);
        self
    }

    pub const fn role_request(&self) -> ProductRoleRequest {
        self.role
    }
}
