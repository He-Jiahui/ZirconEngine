mod capability;
mod plugin;

pub use capability::{
    CONTAINER_IMPORTER_CAPABILITY, IMPORTER_FAMILY, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST,
    PLUGIN_ID, PSD_IMPORTER_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME, TEXTURE_ASSET_IMPORTER_DECLARATION,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, TextureAssetImporterRuntimePlugin,
    TEXTURE_ASSET_IMPORTER_DIST_CRATE_NAME, TEXTURE_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
