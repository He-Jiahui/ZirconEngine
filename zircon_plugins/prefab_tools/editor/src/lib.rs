mod authoring;
mod capability;
mod extension_ids;
mod plugin;

pub use authoring::{effective_prefab_overrides, validate_prefab_instance};
pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{PREFAB_AUTHORING_VIEW_ID, PREFAB_DRAWER_ID, PREFAB_TEMPLATE_ID};
pub use plugin::{
    PrefabToolsEditorPlugin, editor_capabilities, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration,
};

#[cfg(test)]
mod tests;
