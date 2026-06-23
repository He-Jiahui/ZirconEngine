mod capability;
mod plugin;

pub use capability::{RUNTIME_CAPABILITIES, TILEMAP_2D_RUNTIME_CAPABILITY};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, tilemap_component_descriptor,
    tilemap_importer_descriptors, Tilemap2dRuntimePlugin, PLUGIN_ID, TILED_IMPORTER_ID,
    TILEMAP_COMPONENT_TYPE,
};

#[cfg(test)]
mod tests;
