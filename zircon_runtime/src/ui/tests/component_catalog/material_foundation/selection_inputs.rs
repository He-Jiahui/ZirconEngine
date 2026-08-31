use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventKind, UiHostCapability, UiValue,
};

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
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::Focus,
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(select, event);
    }

    let dropdown = registry
        .descriptor("Dropdown")
        .expect("Dropdown descriptor");
    for event in [
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(dropdown, event);
    }

    let dropdown_popup = registry
        .descriptor("DropdownPopup")
        .expect("DropdownPopup descriptor");
    for prop in [
        "open",
        "popup_open",
        "options",
        "selected_options",
        "selectedOptions",
        "disabled_options",
        "focused_options",
        "hovered_options",
        "pressed_options",
        "focused_index",
        "keyboard_navigation",
        "typeahead_buffer",
        "typeahead_buffer_expired",
        "typeahead_timeout_ms",
        "hovered_option_id",
        "submenu_pending_option_id",
        "submenu_open_option_id",
        "submenu_hover_ready",
        "submenu_hover_delay_ms",
        "submenu_focus_scope",
        "submenu_focus_loop",
        "placement",
        "popup_anchor_x",
        "popup_anchor_y",
        "popup_anchor_width",
        "popup_anchor_height",
        "anchor_origin_vertical",
        "anchor_origin_horizontal",
        "transform_origin_vertical",
        "transform_origin_horizontal",
        "popup_offset_x",
        "popup_offset_y",
        "disable_auto_focus",
        "disable_enforce_focus",
        "disable_restore_focus",
        "disable_escape_key_down",
        "close_on_backdrop_click",
        "keep_mounted",
        "aria_modal",
        "z_index",
        "disable_portal",
        "portal_layer",
    ] {
        assert_has_prop(dropdown_popup, prop);
    }
    for slot_name in [
        "paper",
        "listbox",
        "transition",
        "option",
        "groupLabel",
        "groupUl",
    ] {
        assert_has_slot(dropdown_popup, slot_name);
    }
    for event in [
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::KeyboardText,
        UiComponentEventKind::TypeaheadExpired,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Focus,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
    ] {
        assert_has_event(dropdown_popup, event);
    }
    assert_default_value(
        dropdown_popup,
        "placement",
        UiValue::Enum("bottom-start".to_string()),
    );
    assert_default_value(dropdown_popup, "popup_open", UiValue::Bool(false));
    assert_default_value(
        dropdown_popup,
        "close_on_backdrop_click",
        UiValue::Bool(true),
    );

    let autocomplete = registry
        .descriptor("Autocomplete")
        .expect("Autocomplete descriptor");
    for prop in [
        "query",
        "inputValue",
        "value",
        "value_text",
        "selected_options",
        "selectedOptions",
        "options",
        "filtered_options",
        "filteredOptions",
        "disabled_options",
        "disabledOptions",
        "focused_options",
        "focusedOptions",
        "hovered_options",
        "hoveredOptions",
        "pressed_options",
        "pressedOptions",
        "matched_options",
        "matchedOptions",
        "size",
        "multiple",
        "free_solo",
        "freeSolo",
        "popup_open",
        "popupOpen",
        "fullWidth",
        "disableClearable",
        "disablePortal",
        "inputFocused",
        "loading",
        "readOnly",
        "forcePopupIcon",
    ] {
        assert_has_prop(autocomplete, prop);
    }
    for slot_name in [
        "inputRoot",
        "input",
        "tag",
        "endAdornment",
        "clearIndicator",
        "popupIndicator",
        "popper",
        "paper",
        "listbox",
        "loading",
        "noOptions",
        "option",
        "groupLabel",
        "groupUl",
    ] {
        assert!(
            autocomplete.slot_schema(slot_name).is_some(),
            "Autocomplete should declare local MUI slot `{slot_name}`"
        );
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
        ("freeSolo", false),
        ("popup_open", false),
        ("popupOpen", false),
        ("fullWidth", false),
        ("disableClearable", false),
        ("disablePortal", false),
        ("inputFocused", false),
        ("loading", false),
        ("readOnly", false),
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
        UiComponentEventKind::KeyboardAction,
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

    let toggle_button_group = registry
        .descriptor("ToggleButtonGroup")
        .expect("ToggleButtonGroup descriptor");
    for prop in [
        "selection_state",
        "options",
        "value",
        "value_text",
        "selected_index",
        "focused_index",
        "disabled_options",
        "selection_follows_focus",
        "keyboard_navigation",
    ] {
        assert_has_prop(toggle_button_group, prop);
    }
    assert!(
        toggle_button_group.slot_schema("buttons").is_some(),
        "ToggleButtonGroup should expose its grouped buttons slot"
    );
    for event in [
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::Focus,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(toggle_button_group, event);
    }
}

fn assert_has_slot(descriptor: &UiComponentDescriptor, slot_name: &str) {
    assert!(
        descriptor
            .slot_schema
            .iter()
            .any(|slot| slot.name == slot_name),
        "{} missing slot `{slot_name}`",
        descriptor.id
    );
}

fn assert_default_value(descriptor: &UiComponentDescriptor, prop_name: &str, expected: UiValue) {
    assert_eq!(
        descriptor
            .prop(prop_name)
            .and_then(|prop| prop.default_value.as_ref()),
        Some(&expected),
        "{} should default `{prop_name}` to {expected:?}",
        descriptor.id
    );
}
