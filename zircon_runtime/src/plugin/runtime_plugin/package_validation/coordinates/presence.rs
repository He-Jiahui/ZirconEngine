mod completeness;
mod fields;

use crate::plugin::PluginPackageManifest;

use self::{
    completeness::validate_runtime_plugin_package_coordinate_completeness,
    fields::RuntimePluginPackageCoordinateFields,
};

pub(super) fn validate_runtime_plugin_package_coordinate_presence(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) -> bool {
    let fields = RuntimePluginPackageCoordinateFields::from_manifest(package_manifest);
    validate_runtime_plugin_package_coordinate_completeness(&fields, diagnostics);

    fields.declares_any()
}
