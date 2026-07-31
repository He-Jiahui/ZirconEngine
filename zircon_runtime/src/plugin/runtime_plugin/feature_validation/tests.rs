use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

use super::{
    begin_feature_projection_build_observation, observed_embedded_feature_projection_views,
    observed_standalone_feature_projection_builds, validate_runtime_plugin_feature_manifest,
};

#[test]
fn standalone_feature_validation_builds_exactly_one_local_projection() {
    let feature =
        PluginFeatureBundleManifest::new("validation.feature", "Validation Feature", "validation")
            .with_capability("validation.feature.capability")
            .with_dependency(PluginFeatureDependency::primary(
                "validation_provider",
                "validation.provider.capability",
            ))
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "validation.feature.runtime",
                    "validation_feature_runtime",
                )
                .with_capabilities(["validation.feature.module_capability"]),
            );
    let mut diagnostics = Vec::new();

    begin_feature_projection_build_observation();
    validate_runtime_plugin_feature_manifest(&feature, &mut diagnostics);

    assert_eq!(observed_standalone_feature_projection_builds(), 1);
    assert_eq!(observed_embedded_feature_projection_views(), 0);
    assert!(diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.contains("must be unique")));
}
