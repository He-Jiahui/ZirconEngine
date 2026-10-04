use super::shape::{validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_token};

/// 检查实现该功能的提供包 ID 的格式；它可以不同于功能归属插件。
/// 提供包是否存在、已启用及角色是否允许进入产品目录，由注册和目录选择层另行判断。
pub(super) fn validate_runtime_plugin_feature_provider_package_id(
    provider_package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field("provider_package_id", provider_package_id, diagnostics);
    validate_runtime_plugin_feature_token("provider_package_id", provider_package_id, diagnostics);
}
