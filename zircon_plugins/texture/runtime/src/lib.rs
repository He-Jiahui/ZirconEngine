mod capability;
mod manager;
mod module;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES,
    TEXTURE_PLUGIN_DECLARATION, TEXTURE_RUNTIME_CAPABILITY, TEXTURE_RUNTIME_CRATE_NAME,
};
pub use manager::{DefaultTextureManager, TextureImportSummary};
pub use module::{module_descriptor, TEXTURE_MANAGER_NAME, TEXTURE_MODULE_NAME};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, TextureRuntimePlugin, TEXTURE_DIST_CRATE_NAME,
    TEXTURE_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests;
