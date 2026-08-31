use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ProjectPluginManifest, RuntimeProfileId,
};
use zircon_runtime::core::framework::render::RenderProfileBundle;
use zircon_runtime::core::framework::window::WindowDescriptor;

use super::{ProductConfigSource, ProductRoleRequest};

/// Unresolved product-host request collected before module composition or host startup.
#[derive(Clone, Debug, PartialEq)]
pub struct EntryConfig {
    pub(super) role: ProductRoleRequest,
    pub(super) runtime_profile: Option<RuntimeProfileId>,
    pub(super) target_mode: Option<RuntimeTargetMode>,
    pub(super) project_plugins: Option<ProjectPluginManifest>,
    pub(super) project_plugins_source: Option<ProductConfigSource>,
    pub(super) export_profile: Option<ExportProfile>,
    pub(super) render_profile: Option<RenderProfileBundle>,
    pub(super) window_descriptor: Option<WindowDescriptor>,
    pub(super) editor_enabled_subsystems: Option<Vec<String>>,
    pub(super) editor_runtime_sandbox_enabled: Option<bool>,
    pub(super) required_runtime_plugins: Vec<RuntimePluginId>,
    pub(super) optional_runtime_plugins: Vec<RuntimePluginId>,
}
