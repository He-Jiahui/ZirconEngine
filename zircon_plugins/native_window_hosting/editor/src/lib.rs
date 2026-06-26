mod capability;
mod extension_ids;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{
    NATIVE_WINDOW_DRAWER_ID, NATIVE_WINDOW_TEMPLATE_ID, PREFAB_WINDOW_VIEW_ID,
    WORKBENCH_WINDOW_VIEW_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor,
    native_window_hosting_dist_module_manifest, package_manifest, plugin_registration,
    NativeWindowHostingEditorPlugin, NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME,
    NATIVE_WINDOW_HOSTING_DIST_EDITOR_ENTRY,
};

#[cfg(test)]
mod tests;
