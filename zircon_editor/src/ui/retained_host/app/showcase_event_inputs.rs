use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

mod action;
mod edit;

pub(super) use action::demo_input_for_showcase_action;
pub(super) use edit::demo_input_for_showcase_edit;

const DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT: i64 = 36;
const DEFAULT_PAGED_LIST_PAGE_SIZE: i64 = 100;

fn action_matches(action_id: &str, needle: &str) -> bool {
    action_key(action_id).contains(needle)
}

fn action_matches_binding_suffix(action_id: &str, binding_suffix: &str) -> bool {
    action_key(action_id).contains(&camel_to_snake_segment(binding_suffix))
}

fn action_key(action_id: &str) -> String {
    action_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
        .map(camel_to_snake_segment)
        .collect::<Vec<_>>()
        .join(".")
}

fn camel_to_snake_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
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

pub(super) fn select_option(option_id: &str, selected: bool) -> UiComponentShowcaseDemoEventInput {
    UiComponentShowcaseDemoEventInput::SelectOption {
        option_id: option_id.to_string(),
        selected,
    }
}
