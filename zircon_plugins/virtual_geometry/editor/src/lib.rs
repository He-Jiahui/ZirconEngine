mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, PLUGIN_ID, VIRTUAL_GEOMETRY_AUTHORING_CAPABILITY};
pub use extension_ids::{
    VIRTUAL_GEOMETRY_AUTHORING_VIEW_ID, VIRTUAL_GEOMETRY_DRAWER_ID, VIRTUAL_GEOMETRY_TEMPLATE_ID,
};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, VirtualGeometryEditorPlugin,
};
