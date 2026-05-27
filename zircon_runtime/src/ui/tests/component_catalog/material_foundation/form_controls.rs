use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiHostCapability, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_button_base(registry);
    assert_text_input_variants(registry);
    assert_form_control_stack(registry);
    assert_native_select(registry);
    assert_radio_group(registry);
    assert_scoped_css_baseline(registry);
}

fn assert_button_base(registry: &UiComponentDescriptorRegistry) {
    let descriptor = registry
        .descriptor("ButtonBase")
        .expect("ButtonBase descriptor");
    for prop in [
        "component",
        "centerRipple",
        "disableRipple",
        "disableTouchRipple",
        "focusRipple",
        "focusVisibleClassName",
        "TouchRippleProps",
    ] {
        assert_has_prop(descriptor, prop);
    }
    assert!(descriptor.slot_schema("touchRipple").is_some());
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::Press,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(descriptor, event);
    }
}

fn assert_text_input_variants(registry: &UiComponentDescriptorRegistry) {
    for component_id in ["Input", "InputBase", "FilledInput", "OutlinedInput"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("{component_id} descriptor"));
        for prop in [
            "value",
            "value_text",
            "placeholder",
            "type",
            "autoFocus",
            "readOnly",
            "inputReadOnly",
            "fullWidth",
            "formControl",
            "multiline",
            "startAdornment",
            "endAdornment",
        ] {
            assert_has_prop(descriptor, prop);
        }
        assert!(descriptor.slot_schema("input").is_some());
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
        assert_has_event(descriptor, UiComponentEventKind::Commit);
        assert!(descriptor
            .required_host_capabilities
            .contains(&UiHostCapability::TextInput));
    }

    let filled = registry
        .descriptor("FilledInput")
        .expect("FilledInput descriptor");
    assert_has_prop(filled, "disableUnderline");
    assert_eq!(
        filled
            .prop("variant")
            .and_then(|prop| prop.default_value.as_ref()),
        Some(&UiValue::String("filled".to_string()))
    );

    let outlined = registry
        .descriptor("OutlinedInput")
        .expect("OutlinedInput descriptor");
    assert_has_prop(outlined, "label");
    assert_has_prop(outlined, "notched");
    assert!(outlined.slot_schema("notchedOutline").is_some());
    assert_eq!(
        outlined
            .prop("variant")
            .and_then(|prop| prop.default_value.as_ref()),
        Some(&UiValue::String("outlined".to_string()))
    );
}

fn assert_form_control_stack(registry: &UiComponentDescriptorRegistry) {
    let form_control = registry
        .descriptor("FormControl")
        .expect("FormControl descriptor");
    assert_enum_options(form_control, "variant", &["standard", "filled", "outlined"]);
    assert_enum_options(form_control, "margin", &["dense", "none", "normal"]);
    for prop in ["color", "size", "fullWidth", "hiddenLabel", "required"] {
        assert_has_prop(form_control, prop);
    }
    for slot in ["label", "input", "helperText"] {
        assert!(form_control.slot_schema(slot).is_some());
    }

    let form_control_label = registry
        .descriptor("FormControlLabel")
        .expect("FormControlLabel descriptor");
    assert_enum_options(
        form_control_label,
        "labelPlacement",
        &["bottom", "end", "start", "top"],
    );
    for slot in ["control", "label", "asterisk"] {
        assert!(form_control_label.slot_schema(slot).is_some());
    }
    assert_has_event(form_control_label, UiComponentEventKind::ValueChanged);

    let form_group = registry
        .descriptor("FormGroup")
        .expect("FormGroup descriptor");
    assert_has_prop(form_group, "row");
    assert!(form_group.slot_schema("content").is_some());

    let helper = registry
        .descriptor("FormHelperText")
        .expect("FormHelperText descriptor");
    assert_enum_options(helper, "variant", &["standard", "filled", "outlined"]);
    for prop in ["text", "size", "filled", "required"] {
        assert_has_prop(helper, prop);
    }

    let label = registry
        .descriptor("FormLabel")
        .expect("FormLabel descriptor");
    for prop in ["text", "color", "filled", "required"] {
        assert_has_prop(label, prop);
    }
    assert!(label.slot_schema("asterisk").is_some());

    let adornment = registry
        .descriptor("InputAdornment")
        .expect("InputAdornment descriptor");
    assert_enum_options(adornment, "position", &["end", "start"]);
    for prop in [
        "text",
        "size",
        "variant",
        "disablePointerEvents",
        "disableTypography",
        "hiddenLabel",
    ] {
        assert_has_prop(adornment, prop);
    }

    let input_label = registry
        .descriptor("InputLabel")
        .expect("InputLabel descriptor");
    assert_enum_options(input_label, "variant", &["standard", "filled", "outlined"]);
    for prop in [
        "text",
        "margin",
        "size",
        "disableAnimation",
        "formControl",
        "required",
        "shrink",
    ] {
        assert_has_prop(input_label, prop);
    }
}

fn assert_native_select(registry: &UiComponentDescriptorRegistry) {
    let descriptor = registry
        .descriptor("NativeSelect")
        .expect("NativeSelect descriptor");
    assert_enum_options(descriptor, "variant", &["standard", "filled", "outlined"]);
    for prop in [
        "options",
        "value",
        "defaultValue",
        "multiple",
        "open",
        "inputProps",
        "IconComponent",
    ] {
        assert_has_prop(descriptor, prop);
    }
    for slot in ["select", "icon", "nativeInput"] {
        assert!(descriptor.slot_schema(slot).is_some());
    }
    assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
}

fn assert_radio_group(registry: &UiComponentDescriptorRegistry) {
    let descriptor = registry
        .descriptor("RadioGroup")
        .expect("RadioGroup descriptor");
    for prop in ["options", "value", "defaultValue", "name", "row"] {
        assert_has_prop(descriptor, prop);
    }
    assert!(descriptor.slot_schema("items").is_some());
    assert_has_event(descriptor, UiComponentEventKind::SelectOption);
    assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
}

fn assert_scoped_css_baseline(registry: &UiComponentDescriptorRegistry) {
    let descriptor = registry
        .descriptor("ScopedCssBaseline")
        .expect("ScopedCssBaseline descriptor");
    assert_has_prop(descriptor, "behavior_utility");
    assert_has_prop(descriptor, "enableColorScheme");
    assert_has_prop(descriptor, "component");
    assert!(descriptor.slot_schema("content").is_some());
}
