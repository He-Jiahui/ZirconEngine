mod container;
mod importers;
mod registration;

pub use importers::{import_image, import_psd, import_texture_container};
pub use registration::{
    asset_importer_descriptors, module_descriptor, package_manifest, plugin_registration,
    register_runtime_extensions, runtime_capabilities, runtime_module_manifest, runtime_selection,
    supported_platforms, supported_targets,
};

pub const PLUGIN_ID: &str = "texture_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_texture_importer_runtime";
pub const MODULE_NAME: &str = "TextureImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.texture_importer";
pub const IMAGE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.image";
pub const CONTAINER_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.container";
pub const PSD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.psd";

#[cfg(test)]
mod tests;
