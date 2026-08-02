mod capability;
mod extension_ids;
mod plugin;

pub use capability::{
    CAPABILITY, EDITOR_CAPABILITIES, EDITOR_CRATE_NAME, NATIVE_EDITOR_ENTRY,
    NATIVE_EDITOR_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    PLUGIN_ID, UI_ASSET_AUTHORING_DECLARATION,
};
pub use extension_ids::{UI_ASSET_DRAWER_ID, UI_ASSET_TEMPLATE_ID, UI_ASSET_VIEW_ID};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, ui_asset_authoring_dist_module_manifest, UiAssetAuthoringEditorPlugin,
    UI_ASSET_AUTHORING_DIST_CRATE_NAME, UI_ASSET_AUTHORING_DIST_EDITOR_ENTRY,
};

#[cfg(test)]
mod tests;
