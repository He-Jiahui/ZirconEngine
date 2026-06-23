mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{ANIMATION_AUTHORING_CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{ANIMATION_AUTHORING_VIEW_ID, ANIMATION_DRAWER_ID, ANIMATION_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, AnimationEditorPlugin,
};
