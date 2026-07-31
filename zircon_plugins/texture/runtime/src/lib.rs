mod capability;
mod manager;
mod module;
mod plugin;

pub use capability::{
    PLUGIN_ID, RUNTIME_CAPABILITIES, TEXTURE_PLUGIN_DECLARATION, TEXTURE_RUNTIME_CAPABILITY,
    TEXTURE_RUNTIME_CRATE_NAME,
};
pub use manager::{DefaultTextureManager, TextureImportSummary};
pub use module::{TEXTURE_MANAGER_NAME, TEXTURE_MODULE_NAME, module_descriptor};
pub use plugin::{
    TEXTURE_DIST_CRATE_NAME, TEXTURE_DIST_RUNTIME_ENTRY, TextureRuntimePlugin, package_manifest,
    plugin_registration, runtime_capabilities, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection,
};

#[cfg(test)]
mod tests;
