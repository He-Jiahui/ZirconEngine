use crate::ui::slint_host as host_contract;

pub(super) fn structured_menu_items(
    items: &[String],
) -> Vec<host_contract::TemplatePaneMenuItemData> {
    items
        .iter()
        .map(|item| structured_menu_item(item))
        .collect()
}

fn structured_menu_item(raw: &str) -> host_contract::TemplatePaneMenuItemData {
    if raw == "---" {
        return host_contract::TemplatePaneMenuItemData {
            raw: raw.into(),
            action_id: "".into(),
            label: "".into(),
            shortcut: "".into(),
            checked: false,
            disabled: true,
            separator: true,
            focused: false,
            hovered: false,
            pressed: false,
        };
    }

    let mut parts = raw.splitn(3, '|');
    let label = parts.next().unwrap_or_default().trim();
    let flags = parts.next().unwrap_or_default();
    let shortcut = parts.next().unwrap_or_default().trim();

    host_contract::TemplatePaneMenuItemData {
        raw: raw.into(),
        action_id: label.into(),
        label: label.into(),
        shortcut: shortcut.into(),
        checked: has_flag(flags, "checked"),
        disabled: has_flag(flags, "disabled"),
        separator: false,
        focused: has_flag(flags, "focused"),
        hovered: has_flag(flags, "hovered"),
        pressed: has_flag(flags, "pressed"),
    }
}

fn has_flag(flags: &str, expected: &str) -> bool {
    flags
        .split(',')
        .any(|flag| flag.trim().eq_ignore_ascii_case(expected))
}
