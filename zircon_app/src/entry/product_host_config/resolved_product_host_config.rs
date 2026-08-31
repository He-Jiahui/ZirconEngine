use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ProjectPluginManifest, RuntimeProfileId,
};
use zircon_runtime::core::framework::render::RenderProfileBundle;
use zircon_runtime::core::framework::window::WindowDescriptor;
use zircon_runtime::platform::PlatformTarget;

use crate::entry::EntryProfile;

use super::{ProductHostConfigProvenance, ProductRoleDescriptor, ProductRoleRequest};

/// Immutable product-host contract admitted before Runtime compiles the module composition.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedProductHostConfig {
    pub(super) role: ProductRoleRequest,
    pub(super) role_descriptor: ProductRoleDescriptor,
    pub(super) profile: EntryProfile,
    pub(super) runtime_profile: Option<RuntimeProfileId>,
    pub(super) target_mode: RuntimeTargetMode,
    pub(super) platform_target: PlatformTarget,
    pub(super) project_plugins: Option<ProjectPluginManifest>,
    pub(super) export_profile: Option<ExportProfile>,
    pub(super) render_profile: RenderProfileBundle,
    pub(super) window_descriptor: WindowDescriptor,
    pub(super) editor_enabled_subsystems: Option<Vec<String>>,
    pub(super) editor_runtime_sandbox_enabled: bool,
    pub(super) provenance: ProductHostConfigProvenance,
}

impl ResolvedProductHostConfig {
    pub const fn role(&self) -> ProductRoleRequest {
        self.role
    }

    pub const fn role_descriptor(&self) -> &ProductRoleDescriptor {
        &self.role_descriptor
    }

    pub const fn profile(&self) -> EntryProfile {
        self.profile
    }

    pub const fn runtime_profile(&self) -> Option<RuntimeProfileId> {
        self.runtime_profile
    }

    pub const fn target_mode(&self) -> RuntimeTargetMode {
        self.target_mode
    }

    pub const fn platform_target(&self) -> PlatformTarget {
        self.platform_target
    }

    pub fn project_plugin_manifest(&self) -> Option<&ProjectPluginManifest> {
        self.project_plugins.as_ref()
    }

    pub fn export_profile(&self) -> Option<&ExportProfile> {
        self.export_profile.as_ref()
    }

    pub const fn render_profile(&self) -> &RenderProfileBundle {
        &self.render_profile
    }

    pub const fn window_descriptor(&self) -> &WindowDescriptor {
        &self.window_descriptor
    }

    pub fn editor_enabled_subsystems(&self) -> Option<&[String]> {
        self.editor_enabled_subsystems.as_deref()
    }

    pub const fn editor_runtime_sandbox_enabled(&self) -> bool {
        self.editor_runtime_sandbox_enabled
    }

    pub const fn provenance(&self) -> &ProductHostConfigProvenance {
        &self.provenance
    }
}
