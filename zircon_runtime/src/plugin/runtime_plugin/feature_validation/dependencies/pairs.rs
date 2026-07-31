use crate::plugin::PluginFeatureDependency;

pub(super) fn validate_runtime_plugin_feature_dependency_pair(
    dependency: &PluginFeatureDependency,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin feature manifest dependency `{}` capability `{}` must be unique",
            dependency.plugin_id, dependency.capability
        ));
    }
}
