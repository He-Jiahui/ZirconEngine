mod capability;
mod container;
mod importers;
mod plugin;

pub use capability::{
    CONTAINER_IMPORTER_CAPABILITY, IMAGE_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID,
    PSD_IMPORTER_CAPABILITY, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use importers::{import_image, import_psd, import_texture_container};
pub use plugin::{
    asset_importer_descriptors, module_descriptor, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, TextureImporterRuntimePlugin,
};

#[cfg(test)]
mod tests;
