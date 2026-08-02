mod capability;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES, TERRAIN_DECLARATION,
    TERRAIN_RUNTIME_CAPABILITY,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, terrain_component_descriptor,
    terrain_importer_descriptors, TerrainRuntimePlugin, TERRAIN_COMPONENT_TYPE,
    TERRAIN_DIST_CRATE_NAME, TERRAIN_DIST_RUNTIME_ENTRY, TERRAIN_HEIGHTFIELD_IMPORTER_ID,
};

#[cfg(test)]
mod tests;
