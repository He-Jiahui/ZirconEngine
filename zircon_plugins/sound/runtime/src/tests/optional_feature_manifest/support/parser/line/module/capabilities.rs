// 模块字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
mod field;
mod state;
mod values;

pub(super) fn parse_module_capabilities_line(line: &str, capabilities: &mut Vec<String>) -> bool {
    let Some(value) = field::module_capabilities_value(line) else {
        return false;
    };
    state::set_module_capabilities(
        capabilities,
        values::module_capabilities_from_plugin_toml(value),
    );
    true
}
