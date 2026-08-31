mod capability;
#[cfg(feature = "editor")]
mod plugin;

pub use capability::{
    CAPABILITY, EDITOR_CAPABILITIES, EDITOR_CRATE_NAME, NATIVE_EDITOR_ENTRY,
    NATIVE_EDITOR_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_WINDOW_HOSTING_DECLARATION, PLUGIN_ID,
};
#[cfg(feature = "editor")]
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor,
    native_window_hosting_dist_module_manifest, package_manifest, plugin_registration,
    NativeWindowHostingEditorPlugin, NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME,
    NATIVE_WINDOW_HOSTING_DIST_EDITOR_ENTRY,
};

#[cfg(all(test, feature = "editor"))]
mod tests;
