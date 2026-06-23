mod capability;
mod plugin;

pub use capability::{PREFAB_TOOLS_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
pub use plugin::{
    package_manifest, plugin_registration, prefab_importer_descriptors,
    prefab_instance_component_descriptor, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, PrefabToolsRuntimePlugin,
    PLUGIN_ID, PREFAB_IMPORTER_ID, PREFAB_INSTANCE_COMPONENT_TYPE,
};

#[cfg(test)]
mod tests;
