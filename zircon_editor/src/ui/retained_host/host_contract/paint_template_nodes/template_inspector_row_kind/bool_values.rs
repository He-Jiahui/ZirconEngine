//! 阴影开关的显示值适配静态属性文本，接受现有投影使用的少量英文真值，其他文本显示 Off。
//! 此函数不提交属性修改；编辑与持久化由 workbench property_edit 事件链负责。

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bool_display_value(
    value: &str,
) -> &'static str {
    if bool_value(value) {
        "On"
    } else {
        "Off"
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bool_value(
    value: &str,
) -> bool {
    let value = value.trim();
    match value.len() {
        1 => value == "1",
        2 => value.eq_ignore_ascii_case("on"),
        3 => value.eq_ignore_ascii_case("yes"),
        4 => value.eq_ignore_ascii_case("true"),
        5 => value.eq_ignore_ascii_case("check"),
        7 => value.eq_ignore_ascii_case("checked"),
        _ => false,
    }
}

#[cfg(test)]
#[path = "tests/bool_values.rs"]
mod tests;
