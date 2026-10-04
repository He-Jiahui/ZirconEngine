// 依赖字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
mod capability;
mod plugin_id;

pub(super) fn parse_dependency_identity_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
) -> bool {
    if plugin_id::parse_dependency_plugin_id_line(line, plugin_id) {
        return true;
    }

    capability::parse_dependency_capability_line(line, capability)
}
