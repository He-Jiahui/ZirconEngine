mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, HYBRID_GI_AUTHORING_CAPABILITY, PLUGIN_ID};
pub use extension_ids::{HYBRID_GI_AUTHORING_VIEW_ID, HYBRID_GI_DRAWER_ID, HYBRID_GI_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, HybridGiEditorPlugin,
};
