mod cad;
mod mesh_importer;
mod plugin;

pub use mesh_importer::import_mesh_model;
pub use plugin::{
    asset_importer_descriptors, module_descriptor, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, ModelAssetImporterRuntimePlugin,
};

pub(crate) use mesh_importer::{model_outcome, primitive_from_indexed_mesh};

pub const PLUGIN_ID: &str = "asset_importer.model";
pub const IMPORTER_FAMILY: &str = "model";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_asset_importer_model_runtime";
pub const MODULE_NAME: &str = "ModelImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.model";
pub const MESH_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.model.mesh";
pub const CAD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.model.cad";

#[cfg(test)]
mod tests;
