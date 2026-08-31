use zircon_runtime_interface::ui::component::UiValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TemplatePopupMenuItemState {
    pub(crate) action_id: String,
    pub(crate) label: String,
    pub(crate) disabled: bool,
    pub(crate) separator: bool,
}

pub(crate) fn string_array_value<'a>(values: impl IntoIterator<Item = &'a str>) -> UiValue {
    UiValue::Array(
        values
            .into_iter()
            .map(|value| UiValue::String(value.to_string()))
            .collect(),
    )
}

pub(crate) fn toml_value_string_list(value: &toml::Value) -> Vec<String> {
    match value {
        toml::Value::Array(values) => values
            .iter()
            .filter_map(toml_value_display_text)
            .filter(|value| !value.is_empty())
            .collect(),
        value => toml_value_display_text(value).into_iter().collect(),
    }
}

pub(crate) fn template_popup_menu_item_state(raw: &str) -> Option<TemplatePopupMenuItemState> {
    if raw == "---" {
        return Some(TemplatePopupMenuItemState {
            action_id: String::new(),
            label: String::new(),
            disabled: true,
            separator: true,
        });
    }

    let mut parts = raw.splitn(3, '|');
    let label = parts.next().unwrap_or_default().trim();
    if label.is_empty() {
        return None;
    }
    let flags = parts.next().unwrap_or_default();
    Some(TemplatePopupMenuItemState {
        action_id: explicit_menu_action_id(flags)
            .map(str::to_string)
            .unwrap_or_else(|| menu_item_action_id(label)),
        label: label.to_string(),
        disabled: menu_item_has_flag(flags, "disabled"),
        separator: false,
    })
}

fn explicit_menu_action_id(flags: &str) -> Option<&str> {
    flags
        .split(',')
        .map(str::trim)
        .find_map(|flag| flag.strip_prefix("action="))
        .filter(|action_id| !action_id.is_empty())
}

pub(crate) fn menu_item_action_id(label: &str) -> String {
    format!("menu.item.{}", label_to_action_segment(label))
}

fn label_to_action_segment(label: &str) -> String {
    let mut output = String::with_capacity(label.len());
    let mut previous_was_separator = true;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.is_empty() && !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    if output.ends_with('_') {
        output.pop();
    }
    output
}

pub(crate) fn menu_item_without_transient_flags(raw: &str) -> String {
    if raw == "---" {
        return raw.to_string();
    }

    let mut parts = raw.splitn(3, '|');
    let label = parts.next().unwrap_or_default().trim();
    let flags = parts.next().unwrap_or_default();
    let shortcut = parts.next().unwrap_or_default().trim();
    let persistent_flags = flags
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .filter(|flag| !matches_transient_menu_item_flag(flag))
        .collect::<Vec<_>>();
    if persistent_flags.is_empty() && shortcut.is_empty() {
        label.to_string()
    } else if shortcut.is_empty() {
        format!("{label}|{}", persistent_flags.join(","))
    } else {
        format!("{label}|{}|{shortcut}", persistent_flags.join(","))
    }
}

pub(crate) fn menu_item_with_checked_state(raw: &str, checked: bool) -> String {
    if raw == "---" {
        return raw.to_string();
    }

    let mut parts = raw.splitn(3, '|');
    let label = parts.next().unwrap_or_default().trim();
    let flags = parts.next().unwrap_or_default();
    let shortcut = parts.next().unwrap_or_default().trim();
    let mut persistent_flags = flags
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .filter(|flag| !flag.eq_ignore_ascii_case("checked"))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if checked {
        persistent_flags.push("checked".to_string());
    }
    if persistent_flags.is_empty() && shortcut.is_empty() {
        label.to_string()
    } else if shortcut.is_empty() {
        format!("{label}|{}", persistent_flags.join(","))
    } else {
        format!("{label}|{}|{shortcut}", persistent_flags.join(","))
    }
}

fn toml_value_display_text(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        toml::Value::Datetime(value) => Some(value.to_string()),
        toml::Value::Array(_) | toml::Value::Table(_) => None,
    }
}

fn matches_transient_menu_item_flag(flag: &str) -> bool {
    ["focused", "hovered", "pressed"]
        .iter()
        .any(|expected| flag.eq_ignore_ascii_case(expected))
}

fn menu_item_has_flag(flags: &str, expected: &str) -> bool {
    flags
        .split(',')
        .any(|flag| flag.trim().eq_ignore_ascii_case(expected))
}

#[cfg(test)]
#[path = "popup_primitives/action_segment_tests.rs"]
mod action_segment_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_menu_item_state_parses_action_label_and_disabled_separator_state() {
        let item = template_popup_menu_item_state("Delete|danger,disabled,icon=trash")
            .expect("menu item should parse");
        assert_eq!(item.action_id, "menu.item.delete");
        assert_eq!(item.label, "Delete");
        assert!(item.disabled);
        assert!(!item.separator);

        let separator =
            template_popup_menu_item_state("---").expect("separator should parse as state");
        assert!(separator.disabled);
        assert!(separator.separator);
        assert!(template_popup_menu_item_state("").is_none());
    }

    #[test]
    fn popup_menu_item_state_prefers_an_explicit_generation_action_id() {
        let item = template_popup_menu_item_state(
            "Create UI Layout|action=menu.item.asset_create.17.3,icon=plus",
        )
        .expect("compiled menu item should parse");

        assert_eq!(item.action_id, "menu.item.asset_create.17.3");
        assert_eq!(item.label, "Create UI Layout");
    }

    #[test]
    fn popup_menu_transient_flag_cleanup_preserves_persistent_flags_and_shortcuts() {
        assert_eq!(
            menu_item_without_transient_flags("Open|hovered,icon=folder,pressed|Ctrl+O"),
            "Open|icon=folder|Ctrl+O"
        );
        assert_eq!(
            menu_item_without_transient_flags("Inspect|focused|Ctrl+I"),
            "Inspect||Ctrl+I"
        );
        assert_eq!(menu_item_without_transient_flags("---"), "---");
    }

    #[test]
    fn popup_menu_checked_state_moves_between_choice_rows_without_losing_flags() {
        assert_eq!(
            menu_item_with_checked_state("Play In Editor|checked,icon=play", false),
            "Play In Editor|icon=play"
        );
        assert_eq!(
            menu_item_with_checked_state("Simulate|icon=play", true),
            "Simulate|icon=play,checked"
        );
        assert_eq!(
            menu_item_with_checked_state("Standalone|disabled,icon=grid", true),
            "Standalone|disabled,icon=grid,checked"
        );
    }
}
