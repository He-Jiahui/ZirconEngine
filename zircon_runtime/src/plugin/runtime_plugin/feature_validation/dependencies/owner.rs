use crate::plugin::PluginFeatureDependency;

pub(super) fn validate_runtime_plugin_feature_primary_dependency_owner(
    dependency: &PluginFeatureDependency,
    owner_plugin_id: &str,
    diagnostics: &mut Vec<String>,
) -> usize {
    if !dependency.primary {
        return 0;
    }
    if dependency.plugin_id != owner_plugin_id {
        diagnostics.push(format!(
            "runtime plugin feature manifest primary dependency `{}` must point to owner_plugin_id `{}`",
            dependency.plugin_id, owner_plugin_id
        ));
    }
    1
}
