use crate::plugin::PluginPackageManifest;

use super::super::row::validate_runtime_plugin_package_embedded_feature_row;

pub(super) fn validate_runtime_plugin_package_optional_feature_list(
    package_manifest: &PluginPackageManifest,
    seen_feature_providers: &mut Vec<(String, String)>,
    diagnostics: &mut Vec<String>,
) {
    for feature in &package_manifest.optional_features {
        validate_runtime_plugin_package_embedded_feature_row(
            "optional feature",
            feature,
            package_manifest,
            seen_feature_providers,
            diagnostics,
        );
    }
}
