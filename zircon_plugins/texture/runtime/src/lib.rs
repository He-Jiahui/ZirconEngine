mod capability;
mod manager;
mod module;
mod plugin;

pub use capability::{RUNTIME_CAPABILITIES, TEXTURE_RUNTIME_CAPABILITY};
pub use manager::{DefaultTextureManager, TextureImportSummary};
pub use module::{module_descriptor, TEXTURE_MANAGER_NAME, TEXTURE_MODULE_NAME};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, TextureRuntimePlugin, PLUGIN_ID,
    TEXTURE_DIST_CRATE_NAME, TEXTURE_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests;
