/// 返回是否可以继续分段内容检查；点的存在只代表结构入口，空段仍由下一层裁定。
pub(super) fn validate_runtime_plugin_feature_namespace_segment_count(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    if value.contains('.') {
        return true;
    }

    diagnostics.push(format!(
        "runtime plugin feature manifest {field_name} `{value}` must use at least two dot-separated namespace segments"
    ));
    false
}
