mod capability;
mod extension_ids;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{UI_ASSET_DRAWER_ID, UI_ASSET_TEMPLATE_ID, UI_ASSET_VIEW_ID};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, ui_asset_authoring_dist_module_manifest, UiAssetAuthoringEditorPlugin,
    UI_ASSET_AUTHORING_DIST_CRATE_NAME, UI_ASSET_AUTHORING_DIST_EDITOR_ENTRY,
};

#[cfg(test)]
mod tests;
