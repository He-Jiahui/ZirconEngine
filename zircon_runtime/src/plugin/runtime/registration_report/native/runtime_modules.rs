use crate::plugin::{PluginModuleKind, PluginPackageManifest, RuntimeExtensionRegistry};

// 仅把 Runtime 模块描述符放入报告扩展表；其他模块仍留在清单投影中。
pub(super) fn register_native_package_runtime_modules(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for module in package_manifest
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
    {
        if let Err(error) = extensions.register_module(module.module_descriptor()) {
            diagnostics.push(error.to_string());
        }
    }
}
