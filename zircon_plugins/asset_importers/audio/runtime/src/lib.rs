mod capability;
mod plugin;

pub use capability::{
    AUDIO_ASSET_IMPORTER_DECLARATION, CODEC_IMPORTER_CAPABILITY, IMPORTER_FAMILY, MODULE_NAME,
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, AudioAssetImporterRuntimePlugin,
    AUDIO_ASSET_IMPORTER_DIST_CRATE_NAME, AUDIO_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
