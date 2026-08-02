mod capability;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES, TILEMAP_2D_DECLARATION,
    TILEMAP_2D_RUNTIME_CAPABILITY,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, tilemap_component_descriptor,
    tilemap_importer_descriptors, Tilemap2dRuntimePlugin, TILED_IMPORTER_ID,
    TILEMAP_2D_DIST_CRATE_NAME, TILEMAP_2D_DIST_RUNTIME_ENTRY, TILEMAP_COMPONENT_TYPE,
};

#[cfg(test)]
mod tests;
