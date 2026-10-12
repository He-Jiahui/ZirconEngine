// 依赖字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
pub(in super::super::super) fn parse_optional_feature_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    super::dispatch::parse_dependency_line(line, plugin_id, capability, primary);
}
