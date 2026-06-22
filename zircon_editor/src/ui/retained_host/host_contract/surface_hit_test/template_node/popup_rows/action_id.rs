use crate::ui::retained_host::primitives::SharedString;

pub(super) fn normalized_menu_row_action_id(action_id: &str, label: &str) -> SharedString {
    if action_id.starts_with("menu.item.") {
        return action_id.into();
    }
    menu_item_action_id(if label.is_empty() { action_id } else { label }).into()
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
