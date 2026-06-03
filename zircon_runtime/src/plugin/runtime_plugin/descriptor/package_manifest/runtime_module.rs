use crate::plugin::PluginModuleManifest;

use super::super::RuntimePluginDescriptor;

pub(super) fn descriptor_runtime_module_manifest(
    descriptor: &RuntimePluginDescriptor,
) -> PluginModuleManifest {
    PluginModuleManifest::runtime(
        format!("{}.runtime", descriptor.package_id),
        descriptor.crate_name.clone(),
    )
    .with_target_modes(descriptor.target_modes.iter().copied())
    .with_capabilities(descriptor.capabilities.iter().cloned())
}
