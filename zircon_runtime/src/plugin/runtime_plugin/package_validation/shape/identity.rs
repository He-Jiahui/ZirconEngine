mod charset;
mod start;
mod underscore;

/// 包标识先满足命名空间字符、首字母和下划线边界，后续所有权比较才有稳定的原始 ID。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_id(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    charset::validate_runtime_plugin_package_id_charset(context, field_name, value, diagnostics);
    start::validate_runtime_plugin_package_id_start(context, field_name, value, diagnostics);
    underscore::validate_runtime_plugin_package_id_underscore(
        context,
        field_name,
        value,
        diagnostics,
    );
}
