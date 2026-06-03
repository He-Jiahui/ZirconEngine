use crate::plugin::{PluginModuleKind, PluginPackageManifest};

pub(super) fn native_package_runtime_crate(
    package_manifest: &PluginPackageManifest,
) -> Option<String> {
    native_package_module_crate(package_manifest, PluginModuleKind::Runtime)
}

pub(super) fn native_package_editor_crate(
    package_manifest: &PluginPackageManifest,
) -> Option<String> {
    native_package_module_crate(package_manifest, PluginModuleKind::Editor)
}

fn native_package_module_crate(
    package_manifest: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Option<String> {
    package_manifest
        .modules
        .iter()
        .find(|module| module.kind == kind)
        .map(|module| module.crate_name.clone())
}
