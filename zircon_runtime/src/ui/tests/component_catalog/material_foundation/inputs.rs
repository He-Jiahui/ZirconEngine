use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::{assert_button_style_schema_with_variant_default, assert_enum_options};

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_number_field(registry);
    assert_checkbox(registry);
    assert_radio(registry);
    assert_switch(registry);
    assert_toggle_button(registry);
}

fn assert_number_field(registry: &UiComponentDescriptorRegistry) {
    let number_field = registry
        .descriptor("NumberField")
        .expect("NumberField descriptor");
    for prop in ["value", "min", "max", "step", "large_step"] {
        assert_has_prop(number_field, prop);
    }
    assert_eq!(number_field.prop("value").unwrap().min, Some(0.0));
    assert_eq!(number_field.prop("value").unwrap().max, Some(100.0));
    assert_eq!(number_field.prop("value").unwrap().step, Some(1.0));
    for (prop, expected) in [
        ("value", 0.0),
        ("min", 0.0),
        ("max", 100.0),
        ("step", 1.0),
        ("large_step", 10.0),
    ] {
        assert_eq!(
            number_field
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Float(expected)),
            "NumberField should declare numeric default `{prop}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::BeginDrag,
        UiComponentEventKind::DragDelta,
        UiComponentEventKind::LargeDragDelta,
        UiComponentEventKind::EndDrag,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(number_field, event);
    }
}

fn assert_checkbox(registry: &UiComponentDescriptorRegistry) {
    let checkbox = registry
        .descriptor("Checkbox")
        .expect("Checkbox descriptor");
    for prop in [
        "text",
        "checked",
        "indeterminate",
        "label_click_toggles",
        "indeterminate_resolves_to_checked",
    ] {
        assert_has_prop(checkbox, prop);
    }
    assert!(
        checkbox.state_prop("indeterminate").is_some(),
        "Checkbox should expose indeterminate as state metadata for the glyph painter"
    );
    for (prop, expected) in [
        ("checked", false),
        ("indeterminate", false),
        ("label_click_toggles", true),
        ("indeterminate_resolves_to_checked", true),
    ] {
        assert_eq!(
            checkbox
                .prop(prop)
                .and_then(|schema| schema.default_value.as_ref()),
            Some(&UiValue::Bool(expected)),
            "Checkbox should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(checkbox, event);
    }
}

fn assert_radio(registry: &UiComponentDescriptorRegistry) {
    let radio = registry.descriptor("Radio").expect("Radio descriptor");
    for prop in [
        "text",
        "checked",
        "group_value",
        "option_id",
        "options",
        "disabled_options",
        "label_click_selects",
        "exclusive_group",
        "keyboard_navigation",
    ] {
        assert_has_prop(radio, prop);
    }
    assert_eq!(
        radio
            .prop("options")
            .unwrap()
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec!["editor", "runtime", "disabled", "qa"],
        "Radio should declare representative single-select option ids"
    );
    assert!(
        radio
            .prop("options")
            .unwrap()
            .options
            .iter()
            .any(|option| option.id == "disabled" && option.disabled),
        "Radio should mark the disabled option in its catalog schema"
    );
    for (prop, expected) in [
        ("checked", false),
        ("label_click_selects", true),
        ("exclusive_group", true),
        ("keyboard_navigation", true),
    ] {
        assert_eq!(
            radio
                .prop(prop)
                .and_then(|schema| schema.default_value.as_ref()),
            Some(&UiValue::Bool(expected)),
            "Radio should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(radio, event);
    }
}

fn assert_toggle_button(registry: &UiComponentDescriptorRegistry) {
    let toggle_button = registry
        .descriptor("ToggleButton")
        .expect("ToggleButton descriptor");
    assert_button_style_schema_with_variant_default(toggle_button, "none", "outlined");
    assert_enum_options(toggle_button, "selection_state", &["exclusive", "multiple"]);
    assert_eq!(
        toggle_button
            .default_props
            .iter()
            .find(|(name, _)| name == "selection_state")
            .map(|(_, value)| value),
        Some(&UiValue::Enum("exclusive".to_string())),
        "ToggleButton should default to exclusive selection semantics"
    );
    assert_has_prop(toggle_button, "checked");
    assert_has_event(toggle_button, UiComponentEventKind::ValueChanged);
}

fn assert_switch(registry: &UiComponentDescriptorRegistry) {
    let switch = registry.descriptor("Switch").expect("Switch descriptor");
    for prop in [
        "text",
        "checked",
        "switch_size",
        "switch_color",
        "label_click_toggles",
        "track_click_toggles",
        "thumb_draggable",
    ] {
        assert_has_prop(switch, prop);
    }
    assert_enum_options(switch, "switch_size", &["small", "medium"]);
    assert_enum_options(switch, "switch_color", &["primary", "default", "error"]);
    for (prop, expected) in [
        ("checked", false),
        ("label_click_toggles", true),
        ("track_click_toggles", true),
        ("thumb_draggable", false),
    ] {
        assert_eq!(
            switch
                .prop(prop)
                .and_then(|schema| schema.default_value.as_ref()),
            Some(&UiValue::Bool(expected)),
            "Switch should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
    ] {
        assert_has_event(switch, event);
    }
}
