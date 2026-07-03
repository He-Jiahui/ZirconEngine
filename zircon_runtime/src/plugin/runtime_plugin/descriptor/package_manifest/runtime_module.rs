use crate::plugin::PluginModuleManifest;

use super::super::RuntimePluginDescriptor;

pub(super) fn descriptor_runtime_module_manifest(
    descriptor: &RuntimePluginDescriptor,
) -> PluginModuleManifest {
    let module_descriptor = descriptor.module_descriptor();
    PluginModuleManifest::runtime(
        module_descriptor.name.clone(),
        descriptor.crate_name.clone(),
    )
    .with_description(module_descriptor.description.clone())
    .with_init_level(module_descriptor.init_level)
    .with_module_dependencies(module_descriptor.module_dependencies.iter().cloned())
    .with_target_modes(descriptor.target_modes.iter().copied())
    .with_capabilities(descriptor.capabilities.iter().cloned())
    .with_system_sets(descriptor.system_sets.iter().cloned())
    .with_system_anchors(descriptor.system_anchors.iter().cloned())
}
