//! Editor-side SDK example plugin package.
//!
//! This crate is the first Plugins 12 skeleton-conformance sample: `lib.rs`
//! stays a thin façade, while capability declarations, plugin registration, and
//! extension registration live in named owner modules.

mod capability;
mod extension_ids;
mod extensions;
mod plugin;

pub use capability::{
    ASSET_FIXTURE_CAPABILITY, CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, WINDOW_CAPABILITY,
};
pub use extension_ids::{
    ASSET_INSPECTOR_VIEW_ID, MODEL_ASSET_KIND, MODEL_IMPORTER_ID, MODEL_IMPORT_SETTINGS_COMPONENT,
    MODEL_IMPORT_SETTINGS_TEMPLATE_ID, WINDOW_VIEW_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, plugin_sdk_examples_dist_module_manifest, ExampleAssetInspectorPlugin,
    ExampleWindowEditorPlugin, PluginSdkExamplesEditorPlugin, PLUGIN_SDK_EXAMPLES_DIST_CRATE_NAME,
    PLUGIN_SDK_EXAMPLES_DIST_EDITOR_ENTRY,
};

#[cfg(test)]
mod tests;
