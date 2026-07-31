mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_ui_component_id_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

pub(super) fn validate_duplicate_ui_components(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, component) in package_manifest.ui_components.iter().enumerate() {
        validate_runtime_plugin_package_ui_component_id_uniqueness(
            component.component_id.as_str(),
            projection.ui_component_id_is_duplicate(index),
            diagnostics,
        );
    }
}
