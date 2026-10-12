use crate::plugin::PluginFeatureBundleManifest;

use super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
    validate_runtime_plugin_feature_token,
};

/// 将功能 ID 绑定到其归属插件的命名空间；显示名只承载展示文本，不参与目录键匹配。
/// 不修正输入字符串，确保诊断、注册报告和后续选择仍引用同一份清单。
pub(super) fn validate_runtime_plugin_feature_identity(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field("feature id", &feature.id, diagnostics);
    validate_runtime_plugin_feature_namespace("feature id", &feature.id, diagnostics);
    validate_runtime_plugin_feature_field("display_name", &feature.display_name, diagnostics);
    validate_runtime_plugin_feature_field("owner_plugin_id", &feature.owner_plugin_id, diagnostics);
    validate_runtime_plugin_feature_token("owner_plugin_id", &feature.owner_plugin_id, diagnostics);
    if !feature_id_has_owner(&feature.owner_plugin_id, &feature.id) {
        diagnostics.push(format!(
            "runtime plugin feature manifest feature id `{}` must be prefixed by owner_plugin_id `{}`",
            feature.id, feature.owner_plugin_id
        ));
    }
}

// 归属关系要求完整命名空间边界，不能将仅共享字母前缀的另一个插件当作 owner。
fn feature_id_has_owner(owner_plugin_id: &str, feature_id: &str) -> bool {
    let feature = feature_id.as_bytes();
    let owner = owner_plugin_id.as_bytes();
    feature.len() > owner.len() && feature.starts_with(owner) && feature[owner.len()] == b'.'
}

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
