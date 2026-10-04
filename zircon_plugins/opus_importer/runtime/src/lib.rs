mod capability;
mod plugin;

pub use capability::{
    MODULE_NAME, NATIVE_IMPORTER_CAPABILITY, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, OPUS_IMPORTER_CAPABILITY,
    OPUS_IMPORTER_DECLARATION, OPUS_IMPORTER_ID, OPUS_IMPORTER_PRIORITY, PLUGIN_ID,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    asset_importer_descriptor, asset_importer_descriptors, dist_module_manifest, module_descriptor,
    package_manifest, plugin_registration, runtime_capabilities, runtime_module_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, supported_platforms,
    supported_targets, OpusImporterRuntimePlugin, OPUS_IMPORTER_DIST_CRATE_NAME,
    OPUS_IMPORTER_DIST_RUNTIME_ENTRY,
};

pub(crate) const MISSING_BACKEND_DIAGNOSTIC: &str =
    "opus import requires a NativeDynamic libopus backend";

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
