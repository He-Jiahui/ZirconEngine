use crate::plugin::PluginFeatureDependency;

/// 返回该行对主依赖总数的贡献；归属错误仍计入数量，供最终检查区分两类问题。
pub(super) fn validate_runtime_plugin_feature_primary_dependency_owner(
    dependency: &PluginFeatureDependency,
    owner_plugin_id: &str,
    diagnostics: &mut Vec<String>,
) -> usize {
    if !dependency.primary {
        return 0;
    }
    // BUG: [CR-PLUGIN-VALIDATION-0100] owner 为 sound、主依赖为 audio 时，目录别名判定接受它，但这里追加归属错误，注册报告随后失败；证据：目录的 alias_owner 回归与扩展合并对诊断的致命处理。
    if dependency.plugin_id != owner_plugin_id {
        diagnostics.push(format!(
            "runtime plugin feature manifest primary dependency `{}` must point to owner_plugin_id `{}`",
            dependency.plugin_id, owner_plugin_id
        ));
    }
    1
}
