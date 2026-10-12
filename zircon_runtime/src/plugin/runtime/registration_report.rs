use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

#[path = "registration_report/native.rs"]
mod native;
#[path = "registration_report/native_package_project.rs"]
mod native_package_projection;
#[path = "registration_report/package_contributions.rs"]
mod package_contributions;
#[path = "registration_report/plugin.rs"]
mod plugin;
#[path = "registration_report/status.rs"]
mod status;
#[path = "registration_report/validation.rs"]
mod validation;

/// 保存一次包注册的清单、项目默认选择、扩展登记和诊断，供目录统一建立投影。
#[derive(Clone, Debug)]
pub struct RuntimePluginRegistrationReport {
    pub package_manifest: PluginPackageManifest,
    pub project_selection: ProjectPluginSelection,
    pub extensions: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}
