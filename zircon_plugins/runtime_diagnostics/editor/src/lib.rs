mod capability;
mod extension_ids;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{
    RUNTIME_DIAGNOSTICS_DRAWER_ID, RUNTIME_DIAGNOSTICS_TEMPLATE_ID, RUNTIME_DIAGNOSTICS_VIEW_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, runtime_diagnostics_dist_module_manifest, RuntimeDiagnosticsEditorPlugin,
    RUNTIME_DIAGNOSTICS_DIST_CRATE_NAME, RUNTIME_DIAGNOSTICS_DIST_EDITOR_ENTRY,
};

#[cfg(test)]
mod tests;
