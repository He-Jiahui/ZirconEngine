use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

#[path = "edit/collections.rs"]
mod collections;
#[path = "edit/lists.rs"]
mod lists;
#[path = "edit/menu.rs"]
mod menu;
#[path = "edit/values.rs"]
mod values;

#[cfg(test)]
#[path = "tests/edit.rs"]
mod tests;

pub(in crate::ui::retained_host::app) fn demo_input_for_showcase_edit(
    action_id: &str,
    value: &str,
) -> UiComponentShowcaseDemoEventInput {
    if let Some(input) = menu::demo_menu_edit_input(action_id, value) {
        return input;
    }
    if let Some(input) = lists::demo_list_edit_input(action_id, value) {
        return input;
    }
    if let Some(input) = collections::demo_collection_edit_input(action_id, value) {
        return input;
    }
    values::demo_value_edit_input(action_id, value)
}
