use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiHostCapability, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    let select = registry.descriptor("Select").expect("Select descriptor");
    assert_enum_options(select, "variant", &["outlined", "filled", "standard"]);
    for prop in [
        "value",
        "value_text",
        "selected_options",
        "label",
        "placeholder",
        "helper_text",
        "options",
        "multiple",
        "display_empty",
        "popup_open",
        "disabled_options",
        "focused_options",
        "hovered_options",
        "pressed_options",
    ] {
        assert_has_prop(select, prop);
    }
    assert_eq!(
        select
            .prop("options")
            .unwrap()
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec!["primary", "secondary", "disabled"],
        "Select should declare representative option ids"
    );
    assert!(
        select
            .prop("options")
            .unwrap()
            .options
            .iter()
            .any(|option| option.id == "disabled" && option.disabled),
        "Select should mark the disabled option in its catalog schema"
    );
    for (prop, expected) in [
        ("multiple", false),
        ("display_empty", false),
        ("popup_open", false),
    ] {
        assert_eq!(
            select
                .prop(prop)
                .and_then(|schema| schema.default_value.as_ref()),
            Some(&UiValue::Bool(expected)),
            "Select should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(select, event);
    }

    let autocomplete = registry
        .descriptor("Autocomplete")
        .expect("Autocomplete descriptor");
    for prop in [
        "query",
        "value",
        "value_text",
        "selected_options",
        "options",
        "filtered_options",
        "disabled_options",
        "focused_options",
        "hovered_options",
        "pressed_options",
        "matched_options",
        "multiple",
        "free_solo",
        "popup_open",
    ] {
        assert_has_prop(autocomplete, prop);
    }
    assert_eq!(
        autocomplete
            .prop("options")
            .unwrap()
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec!["atlas", "asset", "disabled"],
        "Autocomplete should declare representative option ids"
    );
    assert!(
        autocomplete
            .prop("options")
            .unwrap()
            .options
            .iter()
            .any(|option| option.id == "disabled" && option.disabled),
        "Autocomplete should mark the disabled option in its catalog schema"
    );
    for (prop, expected) in [
        ("multiple", false),
        ("free_solo", false),
        ("popup_open", false),
    ] {
        assert_eq!(
            autocomplete
                .prop(prop)
                .and_then(|schema| schema.default_value.as_ref()),
            Some(&UiValue::Bool(expected)),
            "Autocomplete should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::RemoveElement,
    ] {
        assert_has_event(autocomplete, event);
    }
    assert!(autocomplete
        .required_host_capabilities
        .contains(&UiHostCapability::TextInput));
}
