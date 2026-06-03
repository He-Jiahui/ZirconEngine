use crate::plugin::PluginFeatureDependency;

pub(super) fn validate_runtime_plugin_feature_dependency_pair<'a>(
    dependency: &'a PluginFeatureDependency,
    seen: &mut Vec<(&'a str, &'a str)>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&(
        dependency.plugin_id.as_str(),
        dependency.capability.as_str(),
    )) {
        diagnostics.push(format!(
            "runtime plugin feature manifest dependency `{}` capability `{}` must be unique",
            dependency.plugin_id, dependency.capability
        ));
    } else {
        seen.push((
            dependency.plugin_id.as_str(),
            dependency.capability.as_str(),
        ));
    }
}
