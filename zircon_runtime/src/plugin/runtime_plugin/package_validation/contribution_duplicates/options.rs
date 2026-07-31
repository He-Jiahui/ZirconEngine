mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_option_key_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

pub(super) fn validate_duplicate_plugin_options(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, option) in package_manifest.options.iter().enumerate() {
        validate_runtime_plugin_package_option_key_uniqueness(
            option.key.as_str(),
            projection.option_key_is_duplicate(index),
            diagnostics,
        );
    }
}
