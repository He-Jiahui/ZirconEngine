use crate::builtin::RuntimePluginId;
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
    let owner_matches = RuntimePluginId::parse_key(&dependency.plugin_id)
        .zip(RuntimePluginId::parse_key(owner_plugin_id))
        .is_some_and(|(dependency_id, owner_id)| dependency_id == owner_id);
    if !owner_matches {
        diagnostics.push(format!(
            "runtime plugin feature manifest primary dependency `{}` must point to owner_plugin_id `{}`",
            dependency.plugin_id, owner_plugin_id
        ));
    }
    1
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_plugin_feature_primary_dependency_owner;
    use crate::plugin::PluginFeatureDependency;

    #[test]
    fn primary_owner_alias_matches_canonical_plugin_identity() {
        let dependency = PluginFeatureDependency::primary("audio", "audio.mix");
        let mut diagnostics = Vec::new();

        assert_eq!(
            validate_runtime_plugin_feature_primary_dependency_owner(
                &dependency,
                "sound",
                &mut diagnostics,
            ),
            1
        );
        assert!(diagnostics.is_empty());
    }
}
