use super::super::super::projection::{
    EmbeddedFeatureKind, RuntimePluginPackageValidationProjection,
};
use crate::plugin::PluginPackageManifest;

use super::super::row::validate_runtime_plugin_package_embedded_feature_row;

pub(super) fn validate_runtime_plugin_package_feature_extension_list(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (feature_index, feature) in package_manifest.feature_extensions.iter().enumerate() {
        validate_runtime_plugin_package_embedded_feature_row(
            "feature extension",
            feature,
            package_manifest,
            EmbeddedFeatureKind::Extension,
            feature_index,
            projection,
            diagnostics,
        );
    }
}
