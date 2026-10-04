// 模块字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
mod crate_name;
mod name;

pub(super) fn parse_module_identity_line(
    line: &str,
    name: &mut Option<String>,
    crate_name: &mut Option<String>,
) -> bool {
    if name::parse_module_name_line(line, name) {
        return true;
    }

    crate_name::parse_module_crate_name_line(line, crate_name)
}
