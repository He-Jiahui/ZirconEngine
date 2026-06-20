use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

use super::action_matches;

mod collections;
mod fields;
mod lists;
mod references;
mod selection;
mod world_surface;

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;

pub(in crate::ui::retained_host::app) fn demo_input_for_showcase_action(
    control_id: &str,
    action_id: &str,
) -> UiComponentShowcaseDemoEventInput {
    if let Some(input) = fields::demo_field_input(action_id) {
        return input;
    }
    if let Some(input) = selection::demo_selection_input(action_id) {
        return input;
    }
    if let Some(input) = references::demo_reference_input(action_id) {
        return input;
    }
    if let Some(input) = collections::demo_collection_input(action_id) {
        return input;
    }
    if let Some(input) = lists::demo_list_input(action_id) {
        return input;
    }
    if let Some(input) = world_surface::demo_world_surface_input(action_id) {
        return input;
    }
    component_showcase_input(control_id, action_id)
        .unwrap_or(UiComponentShowcaseDemoEventInput::None)
}

fn component_showcase_input(
    control_id: &str,
    action_id: &str,
) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "show") && control_id.starts_with("ComponentShowcase") => {
            Some(UiComponentShowcaseDemoEventInput::None)
        }
        _ => None,
    }
}
