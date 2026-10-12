use crate::plugin::PluginFeatureDependency;

// 相同插件可提供多个依赖能力；只有插件与能力的组合相同才是重复声明，
// 主依赖标记不会使同一组合成为另一条独立依赖。
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
