mod cad;
mod capability;
mod mesh_importer;
mod plugin;

pub use capability::{
    CAD_IMPORTER_CAPABILITY, IMPORTER_FAMILY, MESH_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use mesh_importer::import_mesh_model;
pub use plugin::{
    asset_importer_descriptors, module_descriptor, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, ModelAssetImporterRuntimePlugin,
};

pub(crate) use mesh_importer::{model_outcome, primitive_from_indexed_mesh};

#[cfg(test)]
mod tests;
