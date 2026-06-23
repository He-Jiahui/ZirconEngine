mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, PHYSICS_AUTHORING_CAPABILITY, PLUGIN_ID};
pub use extension_ids::{PHYSICS_AUTHORING_VIEW_ID, PHYSICS_DRAWER_ID, PHYSICS_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, PhysicsEditorPlugin,
};
