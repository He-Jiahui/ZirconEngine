mod array;
mod capability;
mod container;
mod cubemap;
mod importers;
mod manifest_source;
mod plugin;

pub use array::import_texture_array_manifest;
pub use capability::{
    ARRAY_IMPORTER_CAPABILITY, CONTAINER_IMPORTER_CAPABILITY, CUBEMAP_IMPORTER_CAPABILITY,
    IMAGE_IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, PSD_IMPORTER_CAPABILITY,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME, TEXTURE_IMPORTER_DECLARATION,
};
pub use cubemap::import_cubemap_manifest;
pub use importers::{import_image, import_psd, import_texture_container};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    TextureImporterRuntimePlugin, TEXTURE_IMPORTER_DIST_CRATE_NAME,
    TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests;
