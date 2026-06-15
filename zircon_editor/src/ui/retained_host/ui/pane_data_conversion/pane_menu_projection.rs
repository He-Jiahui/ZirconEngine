use crate::ui::retained_host as host_contract;

pub(in crate::ui::retained_host::ui) fn structured_menu_items(
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
            loading: false,
        };
    }

    let mut parts = raw.splitn(3, '|');
    let label = parts.next().unwrap_or_default().trim();
    let flags = parts.next().unwrap_or_default();
    let shortcut = parts.next().unwrap_or_default().trim();

    host_contract::TemplatePaneMenuItemData {
        raw: raw.into(),
        action_id: menu_item_action_id(label).into(),
        label: label.into(),
        shortcut: shortcut.into(),
        checked: has_flag(flags, "checked"),
        disabled: has_flag(flags, "disabled"),
        separator: false,
        focused: has_flag(flags, "focused"),
        hovered: has_flag(flags, "hovered"),
        pressed: has_flag(flags, "pressed"),
        loading: has_flag(flags, "loading"),
    }
}

fn menu_item_action_id(label: &str) -> String {
    format!("menu.item.{}", label_to_action_segment(label))
}

fn label_to_action_segment(label: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

fn has_flag(flags: &str, expected: &str) -> bool {
    flags
        .split(',')
        .any(|flag| flag.trim().eq_ignore_ascii_case(expected))
}
