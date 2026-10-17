#[path = "array.rs"]
mod array;
#[path = "capability.rs"]
mod capability;
#[path = "container/mod.rs"]
mod container;
#[path = "cubemap.rs"]
mod cubemap;
#[path = "importers.rs"]
mod importers;
#[path = "manifest_source.rs"]
mod manifest_source;
#[path = "mipgen/mod.rs"]
mod mipgen;
#[path = "normal_convention.rs"]
mod normal_convention;
#[path = "plugin.rs"]
mod plugin;
#[path = "transcode/mod.rs"]
mod transcode;

pub use array::import_texture_array_manifest;
pub use capability::{
    ARRAY_IMPORTER_CAPABILITY, CONTAINER_IMPORTER_CAPABILITY, CUBEMAP_IMPORTER_CAPABILITY,
    IMAGE_IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, PSD_IMPORTER_CAPABILITY,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME, TEXTURE_IMPORTER_DECLARATION,
};
pub use cubemap::import_cubemap;
pub use importers::{import_image, import_psd, import_texture_container};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    TextureImporterRuntimePlugin, TEXTURE_IMPORTER_DIST_CRATE_NAME,
    TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
