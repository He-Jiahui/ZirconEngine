use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

mod native;
mod native_package_projection;
mod package_contributions;
mod plugin;
mod status;
mod validation;

/// 保存一次包注册的清单、项目默认选择、扩展登记和诊断，供目录统一建立投影。
#[derive(Clone, Debug)]
pub struct RuntimePluginRegistrationReport {
    pub package_manifest: PluginPackageManifest,
    pub project_selection: ProjectPluginSelection,
    pub extensions: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}
