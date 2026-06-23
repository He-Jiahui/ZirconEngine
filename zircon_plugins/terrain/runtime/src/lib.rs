mod capability;
mod plugin;

pub use capability::{RUNTIME_CAPABILITIES, TERRAIN_RUNTIME_CAPABILITY};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, terrain_component_descriptor,
    terrain_importer_descriptors, TerrainRuntimePlugin, PLUGIN_ID, TERRAIN_COMPONENT_TYPE,
    TERRAIN_DIST_CRATE_NAME, TERRAIN_DIST_RUNTIME_ENTRY, TERRAIN_HEIGHTFIELD_IMPORTER_ID,
};

#[cfg(test)]
mod tests;
