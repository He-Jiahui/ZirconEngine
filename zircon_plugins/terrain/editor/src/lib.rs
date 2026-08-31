mod authoring;
mod capability;
mod extension_ids;
mod plugin;

pub use authoring::{
    TerrainHeightfieldImportRequest, TerrainHeightfieldSourceFormat, TerrainImportKind,
    TerrainImportPlan, plan_terrain_import, terrain_import_output_kind,
    validate_heightfield_import,
};
pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{TERRAIN_AUTHORING_VIEW_ID, TERRAIN_DRAWER_ID, TERRAIN_TEMPLATE_ID};
pub use plugin::{
    TerrainEditorPlugin, editor_capabilities, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration,
};

#[cfg(test)]
mod tests;
