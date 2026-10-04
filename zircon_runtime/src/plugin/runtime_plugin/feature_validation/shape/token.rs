mod charset;
mod start;
mod underscore;

/// 提供包、归属插件和依赖插件使用相同的标记契约。
/// 各条规则分别累积诊断；单独调用某个子规则不能替代此完整检查。
pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_token(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    start::validate_runtime_plugin_feature_token_start(field_name, value, diagnostics);
    charset::validate_runtime_plugin_feature_token_charset(field_name, value, diagnostics);
    underscore::validate_runtime_plugin_feature_token_underscore(field_name, value, diagnostics);
}
