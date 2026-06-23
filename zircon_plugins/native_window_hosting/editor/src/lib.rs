mod capability;
mod extension_ids;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{
    NATIVE_WINDOW_DRAWER_ID, NATIVE_WINDOW_TEMPLATE_ID, PREFAB_WINDOW_VIEW_ID,
    WORKBENCH_WINDOW_VIEW_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, NativeWindowHostingEditorPlugin,
};

#[cfg(test)]
mod tests;
