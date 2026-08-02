mod capability;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, PREFAB_TOOLS_DECLARATION,
    PREFAB_TOOLS_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use plugin::{
    package_manifest, plugin_registration, prefab_importer_descriptors,
    prefab_instance_component_descriptor, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, PrefabToolsRuntimePlugin,
    PREFAB_IMPORTER_ID, PREFAB_INSTANCE_COMPONENT_TYPE, PREFAB_TOOLS_DIST_CRATE_NAME,
    PREFAB_TOOLS_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests;
