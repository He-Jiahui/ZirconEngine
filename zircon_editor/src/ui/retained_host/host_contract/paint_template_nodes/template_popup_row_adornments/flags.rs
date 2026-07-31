use super::super::super::data::TemplatePaneMenuItemData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_has_flag(
    item: &TemplatePaneMenuItemData,
    expected: &str,
) -> bool {
    menu_item_flags(item).any(|flag| flag.eq_ignore_ascii_case(expected))
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
