mod feature_extension;
mod standard;

use crate::plugin::{PluginPackageKind, PluginPackageManifest};

pub(super) fn validate_runtime_plugin_package_feature_kind(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    match package_manifest.package_kind {
        PluginPackageKind::Standard => {
            standard::validate_standard_package_feature_kind(package_manifest, diagnostics)
        }
        PluginPackageKind::FeatureExtension => {
            feature_extension::validate_feature_extension_package_feature_kind(
                package_manifest,
                diagnostics,
            );
        }
    }
}
