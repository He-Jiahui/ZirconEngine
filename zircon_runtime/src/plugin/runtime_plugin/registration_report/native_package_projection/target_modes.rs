use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginModuleKind, PluginPackageManifest};

pub(super) fn native_package_target_modes(
    package_manifest: &PluginPackageManifest,
) -> Vec<RuntimeTargetMode> {
    let mut target_modes = Vec::new();
    for target_mode in package_manifest
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    target_modes
}
