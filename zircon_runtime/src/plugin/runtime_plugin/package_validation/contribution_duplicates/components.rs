mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_component_type_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

pub(super) fn validate_duplicate_components(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, component) in package_manifest.components.iter().enumerate() {
        validate_runtime_plugin_package_component_type_uniqueness(
            component.type_id.as_str(),
            projection.component_type_id_is_duplicate(index),
            diagnostics,
        );
    }
}
