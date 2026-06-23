mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, PLUGIN_ID, TEXTURE_AUTHORING_CAPABILITY};
pub use extension_ids::{TEXTURE_AUTHORING_VIEW_ID, TEXTURE_DRAWER_ID, TEXTURE_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, TextureEditorPlugin,
};
