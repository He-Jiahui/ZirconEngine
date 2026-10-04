//! 读取菜单项原始投影flag的声明值；loading/danger只扫描一次，供共享popup样式使用。
//! flag文本是结构化属性投影，不应用菜单显示标签推断危险性。

use super::super::super::data::TemplatePaneMenuItemData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_has_flag(
    item: &TemplatePaneMenuItemData,
    expected: &str,
) -> bool {
    menu_item_flags(item).any(|flag| flag.eq_ignore_ascii_case(expected))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_loading_and_danger(
    item: &TemplatePaneMenuItemData,
) -> (bool, bool) {
    let mut loading = item.loading;
    let mut danger = false;

    for flag in menu_item_flags(item) {
        match flag.len() {
            7 if !loading && flag.eq_ignore_ascii_case("loading") => loading = true,
            6 if !danger && flag.eq_ignore_ascii_case("danger") => danger = true,
            _ => {}
        }
        if loading && danger {
            break;
        }
    }

    (loading, danger)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_flag_value<
    'a,
>(
    item: &'a TemplatePaneMenuItemData,
    expected_key: &str,
) -> Option<&'a str> {
    menu_item_flags(item).find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        let value = value.trim();
        (key.trim().eq_ignore_ascii_case(expected_key) && !value.is_empty()).then_some(value)
    })
}

fn menu_item_flags(item: &TemplatePaneMenuItemData) -> impl Iterator<Item = &str> {
    item.raw
        .as_str()
        .split('|')
        .nth(1)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
}

#[cfg(test)]
#[path = "tests/flags_optimization_batch_es_tests.rs"]
mod optimization_batch_es_tests;
