use crate::plugin::PluginPackageManifest;

pub(super) fn validate_feature_extension_package_feature_kind(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if package_manifest.feature_extensions.is_empty() {
        diagnostics.push(
            "runtime plugin package manifest package_kind FeatureExtension must declare at least one feature_extension"
                .to_string(),
        );
    }
    if !package_manifest.optional_features.is_empty() {
        diagnostics.push(
            "runtime plugin package manifest package_kind FeatureExtension must not declare optional_features"
                .to_string(),
        );
    }
}
