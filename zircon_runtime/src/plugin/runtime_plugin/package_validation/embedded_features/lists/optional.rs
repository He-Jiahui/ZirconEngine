use super::super::super::projection::{
    EmbeddedFeatureKind, RuntimePluginPackageValidationProjection,
};
use crate::plugin::PluginPackageManifest;

use super::super::row::validate_runtime_plugin_package_embedded_feature_row;

pub(super) fn validate_runtime_plugin_package_optional_feature_list(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (feature_index, feature) in package_manifest.optional_features.iter().enumerate() {
        validate_runtime_plugin_package_embedded_feature_row(
            "optional feature",
            feature,
            package_manifest,
            EmbeddedFeatureKind::Optional,
            feature_index,
            projection,
            diagnostics,
        );
    }
}
