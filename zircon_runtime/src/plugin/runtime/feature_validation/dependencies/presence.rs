use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn validate_runtime_plugin_feature_dependency_presence(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    if feature.dependencies.is_empty() {
        diagnostics.push(
            "runtime plugin feature manifest dependencies must declare at least one dependency"
                .to_string(),
        );
    }
}
