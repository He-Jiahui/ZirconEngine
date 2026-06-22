use super::{
    preferred_showcase_action_buttons, preferred_showcase_action_id, showcase_action_id_for_suffix,
};
use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::binding::UiEventKind;

fn binding(binding_id: &str) -> RetainedUiHostBindingProjection {
    RetainedUiHostBindingProjection {
        binding_id: binding_id.to_string(),
        action_id: String::new(),
        event_kind: UiEventKind::Click,
        route_id: None,
    }
}

#[test]
fn showcase_action_id_for_suffix_normalizes_showcase_binding_paths() {
    let bindings = [binding("UiComponentShowcase/ArrayFieldSetElement")];

    let action_id = showcase_action_id_for_suffix(&bindings, "ArrayFieldSetElement");

    assert_eq!(
        action_id.as_str(),
        "ui_component_showcase.array_field_set_element"
    );
}

#[test]
fn preferred_showcase_action_id_selects_popup_open_suffixes() {
    let bindings = [
        binding("UiComponentShowcase/DropdownOpenPopup"),
        binding("UiComponentShowcase/DropdownChanged"),
    ];

    let closed = preferred_showcase_action_id("DropdownDemo", false, &bindings);
    let open = preferred_showcase_action_id("DropdownDemo", true, &bindings);

    assert_eq!(
        closed.as_deref(),
        Some("ui_component_showcase.dropdown_open_popup")
    );
    assert_eq!(
        open.as_deref(),
        Some("ui_component_showcase.dropdown_changed")
    );
}

#[test]
fn preferred_showcase_action_buttons_project_only_bound_actions() {
    let bindings = [
        binding("UiComponentShowcase/MapFieldAddEntry"),
        binding("UiComponentShowcase/MapFieldRemoveEntry"),
    ];

    let actions = preferred_showcase_action_buttons("MapFieldDemo", &bindings);

    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0].label.as_str(), "Add");
    assert_eq!(
        actions[0].action_id.as_str(),
        "ui_component_showcase.map_field_add_entry"
    );
    assert_eq!(actions[1].label.as_str(), "Remove");
}
