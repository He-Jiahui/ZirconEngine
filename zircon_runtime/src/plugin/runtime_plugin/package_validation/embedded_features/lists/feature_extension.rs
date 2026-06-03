use crate::plugin::PluginPackageManifest;

use super::super::row::validate_runtime_plugin_package_embedded_feature_row;

pub(super) fn validate_runtime_plugin_package_feature_extension_list(
    package_manifest: &PluginPackageManifest,
    seen_feature_providers: &mut Vec<(String, String)>,
    diagnostics: &mut Vec<String>,
) {
    for feature in &package_manifest.feature_extensions {
        validate_runtime_plugin_package_embedded_feature_row(
            "feature extension",
            feature,
            package_manifest,
            seen_feature_providers,
            diagnostics,
        );
    }
}
