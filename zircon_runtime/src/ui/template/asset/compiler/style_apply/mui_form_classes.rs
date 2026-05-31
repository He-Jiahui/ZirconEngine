use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, bool_from_attributes_any, pascal_case,
    string_attribute_any, string_from_attributes_any,
};

pub(super) fn append_form_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "ButtonBase" => {
            append_button_base_classes(node, prefix);
            true
        }
        "TextField" => {
            append_text_field_classes(node, prefix);
            true
        }
        "InputBase" => {
            append_input_base_classes(node, prefix);
            true
        }
        "Input" | "FilledInput" | "OutlinedInput" => {
            append_input_variant_classes(node, component, prefix);
            true
        }
        "FormControl" => {
            append_form_control_classes(node, prefix);
            true
        }
        "FormControlLabel" => {
            append_form_control_label_classes(node, prefix);
            true
        }
        "FormGroup" | "RadioGroup" => {
            append_row_class(node, prefix);
            true
        }
        "FormHelperText" => {
            append_form_helper_text_classes(node, prefix);
            true
        }
        "FormLabel" => {
            append_form_label_classes(node, prefix);
            true
        }
        "InputAdornment" => {
            append_input_adornment_classes(node, prefix);
            true
        }
        "InputLabel" => {
            append_input_label_classes(node, prefix);
            true
        }
        "NativeSelect" | "ScopedCssBaseline" => true,
        _ => false,
    }
}

pub(super) fn append_form_slot_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) -> bool {
    match (owner_component, slot_name) {
        ("InputBase" | "Input" | "FilledInput" | "OutlinedInput", "input") => {
            append_input_slot_classes(child, owner_component, owner_attributes);
            true
        }
        ("TextField", "input") => {
            append_text_field_input_slot_classes(child, owner_attributes);
            true
        }
        ("TextField", "htmlInput") => {
            append_text_field_html_input_slot_classes(child, owner_attributes);
            true
        }
        ("TextField", "inputLabel") => {
            append_text_field_input_label_slot_classes(child, owner_attributes);
            true
        }
        ("TextField", "formHelperText") => {
            append_text_field_form_helper_text_slot_classes(child, owner_attributes);
            true
        }
        ("OutlinedInput", "notchedOutline") => true,
        ("FormControlLabel", "label") => {
            if bool_from_attributes_any(owner_attributes, &["disabled"]) {
                append_class(&mut child.classes, "Mui-disabled".to_string());
            }
            true
        }
        ("FormControlLabel" | "FormLabel" | "InputLabel", "asterisk") => {
            if bool_from_attributes_any(owner_attributes, &["error"]) {
                append_class(&mut child.classes, "Mui-error".to_string());
            }
            true
        }
        ("NativeSelect", "select") => {
            append_native_select_select_slot_classes(child, owner_attributes);
            true
        }
        ("NativeSelect", "icon") => {
            append_native_select_icon_slot_classes(child, owner_attributes);
            true
        }
        _ => false,
    }
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(
        component,
        "ButtonBase"
            | "FilledInput"
            | "FormControl"
            | "FormControlLabel"
            | "FormGroup"
            | "FormHelperText"
            | "FormLabel"
            | "Input"
            | "InputAdornment"
            | "InputBase"
            | "InputLabel"
            | "NativeSelect"
            | "OutlinedInput"
            | "RadioGroup"
            | "ScopedCssBaseline"
            | "TextField"
    )
}

fn append_button_base_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "disabled") {
        append_class(&mut node.classes, format!("{prefix}-disabled"));
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible", "focused"]) {
        append_class(&mut node.classes, format!("{prefix}-focusVisible"));
    }
}

fn append_text_field_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_class(&mut node.classes, "MuiFormControl-root".to_string());
    append_form_control_classes(node, "MuiFormControl");
    append_class(
        &mut node.classes,
        format!("{prefix}-{}", text_field_variant(&node.attributes)),
    );
    if string_attribute_any(node, &["size"]).as_deref() == Some("small") {
        append_class(&mut node.classes, format!("{prefix}-sizeSmall"));
    }
    if string_attribute_any(node, &["color"]).as_deref() == Some("secondary") {
        append_class(&mut node.classes, format!("{prefix}-colorSecondary"));
    }
    for (flag, class) in [
        ("disabled", "disabled"),
        ("error", "error"),
        ("focused", "focused"),
        ("fullWidth", "fullWidth"),
        ("required", "required"),
        ("multiline", "multiline"),
    ] {
        if bool_attribute(node, flag) {
            append_class(&mut node.classes, format!("{prefix}-{class}"));
        }
    }
    if bool_attribute_any(node, &["select", "select_mode"]) {
        append_class(&mut node.classes, format!("{prefix}-select"));
    }
}

fn append_input_base_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_input_base_state_classes(node, prefix);
}

fn append_input_variant_classes(node: &mut UiTemplateNode, component: &str, prefix: &str) {
    append_class(&mut node.classes, "MuiInputBase-root".to_string());
    append_input_base_state_classes(node, "MuiInputBase");
    match component {
        "Input" | "FilledInput" if !bool_attribute_any(node, &["disableUnderline"]) => {
            append_class(&mut node.classes, format!("{prefix}-underline"));
        }
        _ => {}
    }
    if component == "FilledInput" {
        append_variant_state_classes(node, prefix, false);
    }
}

fn append_input_base_state_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_prefixed_state_classes(node, prefix, &["disabled", "error", "focused"]);
    if string_attribute_any(node, &["color"]).as_deref() == Some("secondary") {
        append_class(&mut node.classes, format!("{prefix}-colorSecondary"));
    }
    for (flag, class) in [
        ("formControl", "formControl"),
        ("fullWidth", "fullWidth"),
        ("hiddenLabel", "hiddenLabel"),
        ("multiline", "multiline"),
    ] {
        if bool_attribute_any(node, &[flag]) {
            append_class(&mut node.classes, format!("{prefix}-{class}"));
        }
    }
    append_variant_state_classes(node, prefix, false);
    if bool_attribute_any(
        node,
        &["readOnly", "inputReadOnly", "read_only", "input_read_only"],
    ) {
        append_class(&mut node.classes, format!("{prefix}-readOnly"));
    }
}

fn append_variant_state_classes(node: &mut UiTemplateNode, prefix: &str, skip_hidden_label: bool) {
    if string_attribute_any(node, &["size"]).as_deref() == Some("small") {
        append_class(&mut node.classes, format!("{prefix}-sizeSmall"));
    }
    if has_adornment(node, "startAdornment") {
        append_class(&mut node.classes, format!("{prefix}-adornedStart"));
    }
    if has_adornment(node, "endAdornment") {
        append_class(&mut node.classes, format!("{prefix}-adornedEnd"));
    }
    if !skip_hidden_label && bool_attribute_any(node, &["hiddenLabel"]) {
        append_class(&mut node.classes, format!("{prefix}-hiddenLabel"));
    }
}

fn append_form_control_classes(node: &mut UiTemplateNode, prefix: &str) {
    let margin = string_attribute_any(node, &["margin"]).unwrap_or_else(|| "none".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-margin{}", pascal_case(&margin)),
    );
    if bool_attribute(node, "fullWidth") {
        append_class(&mut node.classes, format!("{prefix}-fullWidth"));
    }
}

fn append_form_control_label_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_prefixed_state_classes(node, prefix, &["disabled", "error"]);
    if let Some(placement) = string_attribute_any(node, &["labelPlacement"])
        .filter(|value| matches!(value.as_str(), "start" | "top" | "bottom"))
    {
        append_class(
            &mut node.classes,
            format!("{prefix}-labelPlacement{}", pascal_case(&placement)),
        );
    }
    if bool_attribute(node, "required") {
        append_class(&mut node.classes, format!("{prefix}-required"));
    }
}

fn append_row_class(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "row") {
        append_class(&mut node.classes, format!("{prefix}-row"));
    }
}

fn append_form_helper_text_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_prefixed_state_classes(node, prefix, &["disabled", "error", "focused"]);
    if string_attribute_any(node, &["size"]).as_deref() == Some("small") {
        append_class(&mut node.classes, format!("{prefix}-sizeSmall"));
    }
    if string_attribute_any(node, &["variant"])
        .is_some_and(|value| matches!(value.as_str(), "filled" | "outlined"))
    {
        append_class(&mut node.classes, format!("{prefix}-contained"));
    }
    for (flag, class) in [("filled", "filled"), ("required", "required")] {
        if bool_attribute(node, flag) {
            append_class(&mut node.classes, format!("{prefix}-{class}"));
        }
    }
}

fn append_form_label_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_prefixed_state_classes(node, prefix, &["disabled", "error", "focused"]);
    if string_attribute_any(node, &["color"]).as_deref() == Some("secondary") {
        append_class(&mut node.classes, format!("{prefix}-colorSecondary"));
    }
    for (flag, class) in [("filled", "filled"), ("required", "required")] {
        if bool_attribute(node, flag) {
            append_class(&mut node.classes, format!("{prefix}-{class}"));
        }
    }
}

fn append_input_adornment_classes(node: &mut UiTemplateNode, prefix: &str) {
    let position = string_attribute_any(node, &["position"]).unwrap_or_else(|| "end".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-position{}", pascal_case(&position)),
    );
    let variant =
        string_attribute_any(node, &["variant"]).unwrap_or_else(|| "standard".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));
    if bool_attribute_any(node, &["disablePointerEvents", "disable_pointer_events"]) {
        append_class(&mut node.classes, format!("{prefix}-disablePointerEvents"));
    }
    if bool_attribute(node, "hiddenLabel") {
        append_class(&mut node.classes, format!("{prefix}-hiddenLabel"));
    }
    if string_attribute_any(node, &["size"]).as_deref() == Some("small") {
        append_class(&mut node.classes, format!("{prefix}-sizeSmall"));
    }
}

fn append_input_label_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_prefixed_state_classes(node, prefix, &["disabled", "error", "focused"]);
    if bool_attribute(node, "formControl") {
        append_class(&mut node.classes, format!("{prefix}-formControl"));
    }
    if string_attribute_any(node, &["size"]).as_deref() == Some("small") {
        append_class(&mut node.classes, format!("{prefix}-sizeSmall"));
    }
    if bool_attribute(node, "shrink") {
        append_class(&mut node.classes, format!("{prefix}-shrink"));
    }
    if !bool_attribute_any(node, &["disableAnimation", "disable_animation"]) {
        append_class(&mut node.classes, format!("{prefix}-animated"));
    }
    let variant =
        string_attribute_any(node, &["variant"]).unwrap_or_else(|| "outlined".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));
    if bool_attribute(node, "required") {
        append_class(&mut node.classes, format!("{prefix}-required"));
    }
}

fn append_input_slot_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if owner_component != "InputBase" {
        append_class(&mut child.classes, "MuiInputBase-input".to_string());
    }
    if string_from_attributes_any(owner_attributes, &["type"]).as_deref() == Some("search") {
        append_class(
            &mut child.classes,
            "MuiInputBase-inputTypeSearch".to_string(),
        );
    }
    if bool_from_attributes_any(owner_attributes, &["readOnly", "inputReadOnly"]) {
        append_class(&mut child.classes, "Mui-readOnly".to_string());
    }
}

fn append_text_field_input_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    let variant = text_field_variant(owner_attributes);
    match variant.as_str() {
        "filled" => append_class(&mut child.classes, "MuiFilledInput-root".to_string()),
        "standard" => append_class(&mut child.classes, "MuiInput-root".to_string()),
        _ => append_class(&mut child.classes, "MuiOutlinedInput-root".to_string()),
    }
    append_class(&mut child.classes, "MuiInputBase-root".to_string());
    append_input_owner_state_classes(child, owner_attributes, "MuiInputBase");
    match variant.as_str() {
        "filled" => {
            append_input_owner_state_classes(child, owner_attributes, "MuiFilledInput");
            if !bool_from_attributes_any(owner_attributes, &["disableUnderline"]) {
                append_class(&mut child.classes, "MuiFilledInput-underline".to_string());
            }
        }
        "standard" => {
            append_input_owner_state_classes(child, owner_attributes, "MuiInput");
            if !bool_from_attributes_any(owner_attributes, &["disableUnderline"]) {
                append_class(&mut child.classes, "MuiInput-underline".to_string());
            }
        }
        _ => append_input_owner_state_classes(child, owner_attributes, "MuiOutlinedInput"),
    }
}

fn append_text_field_html_input_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    let variant = text_field_variant(owner_attributes);
    append_class(&mut child.classes, "MuiInputBase-input".to_string());
    match variant.as_str() {
        "filled" => append_class(&mut child.classes, "MuiFilledInput-input".to_string()),
        "standard" => append_class(&mut child.classes, "MuiInput-input".to_string()),
        _ => append_class(&mut child.classes, "MuiOutlinedInput-input".to_string()),
    }
    if string_from_attributes_any(owner_attributes, &["type"]).as_deref() == Some("search") {
        append_class(
            &mut child.classes,
            "MuiInputBase-inputTypeSearch".to_string(),
        );
    }
    if bool_from_attributes_any(owner_attributes, &["readOnly", "inputReadOnly"]) {
        append_class(&mut child.classes, "Mui-readOnly".to_string());
    }
}

fn append_text_field_input_label_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiInputLabel-root".to_string());
    append_class(
        &mut child.classes,
        format!("MuiInputLabel-{}", text_field_variant(owner_attributes)),
    );
    for (flag, class) in [
        ("disabled", "disabled"),
        ("error", "error"),
        ("focused", "focused"),
        ("required", "required"),
    ] {
        if bool_from_attributes_any(owner_attributes, &[flag]) {
            append_class(&mut child.classes, format!("MuiInputLabel-{class}"));
        }
    }
    if string_from_attributes_any(owner_attributes, &["size"]).as_deref() == Some("small") {
        append_class(&mut child.classes, "MuiInputLabel-sizeSmall".to_string());
    }
    if text_field_label_shrinks(owner_attributes) {
        append_class(&mut child.classes, "MuiInputLabel-shrink".to_string());
    }
    append_class(&mut child.classes, "MuiInputLabel-animated".to_string());
}

fn append_text_field_form_helper_text_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiFormHelperText-root".to_string());
    let variant = text_field_variant(owner_attributes);
    if matches!(variant.as_str(), "filled" | "outlined") {
        append_class(
            &mut child.classes,
            "MuiFormHelperText-contained".to_string(),
        );
    }
    for (flag, class) in [
        ("disabled", "disabled"),
        ("error", "error"),
        ("focused", "focused"),
        ("required", "required"),
    ] {
        if bool_from_attributes_any(owner_attributes, &[flag]) {
            append_class(&mut child.classes, format!("MuiFormHelperText-{class}"));
        }
    }
    if string_from_attributes_any(owner_attributes, &["size"]).as_deref() == Some("small") {
        append_class(
            &mut child.classes,
            "MuiFormHelperText-sizeSmall".to_string(),
        );
    }
}

fn append_native_select_select_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    let variant = string_from_attributes_any(owner_attributes, &["variant"])
        .unwrap_or_else(|| "standard".to_string());
    append_class(&mut child.classes, format!("MuiNativeSelect-{variant}"));
    if bool_from_attributes_any(owner_attributes, &["multiple"]) {
        append_class(&mut child.classes, "MuiNativeSelect-multiple".to_string());
    }
}

fn append_native_select_icon_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    let variant = string_from_attributes_any(owner_attributes, &["variant"])
        .unwrap_or_else(|| "standard".to_string());
    append_class(
        &mut child.classes,
        format!("MuiNativeSelect-icon{}", pascal_case(&variant)),
    );
    if bool_from_attributes_any(owner_attributes, &["open", "popup_open"]) {
        append_class(&mut child.classes, "MuiNativeSelect-iconOpen".to_string());
    }
}

fn append_input_owner_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    prefix: &str,
) {
    for (flag, class) in [
        ("disabled", "disabled"),
        ("error", "error"),
        ("focused", "focused"),
        ("fullWidth", "fullWidth"),
        ("multiline", "multiline"),
    ] {
        if bool_from_attributes_any(owner_attributes, &[flag]) {
            append_class(&mut child.classes, format!("{prefix}-{class}"));
        }
    }
    if string_from_attributes_any(owner_attributes, &["size"]).as_deref() == Some("small") {
        append_class(&mut child.classes, format!("{prefix}-sizeSmall"));
    }
    if string_from_attributes_any(owner_attributes, &["color"]).as_deref() == Some("secondary") {
        append_class(&mut child.classes, format!("{prefix}-colorSecondary"));
    }
    if has_adornment_from_attributes(owner_attributes, "startAdornment") {
        append_class(&mut child.classes, format!("{prefix}-adornedStart"));
    }
    if has_adornment_from_attributes(owner_attributes, "endAdornment") {
        append_class(&mut child.classes, format!("{prefix}-adornedEnd"));
    }
    if bool_from_attributes_any(owner_attributes, &["hiddenLabel"]) {
        append_class(&mut child.classes, format!("{prefix}-hiddenLabel"));
    }
    if bool_from_attributes_any(owner_attributes, &["readOnly", "inputReadOnly"]) {
        append_class(&mut child.classes, format!("{prefix}-readOnly"));
    }
}

fn has_adornment(node: &UiTemplateNode, name: &str) -> bool {
    string_attribute_any(node, &[name]).is_some_and(|value| !value.is_empty())
        || bool_attribute(node, name)
}

fn has_adornment_from_attributes(attributes: &BTreeMap<String, Value>, name: &str) -> bool {
    string_from_attributes_any(attributes, &[name]).is_some_and(|value| !value.is_empty())
        || bool_from_attributes_any(attributes, &[name])
}

fn text_field_variant(attributes: &BTreeMap<String, Value>) -> String {
    string_from_attributes_any(attributes, &["variant", "mui_variant"])
        .filter(|variant| matches!(variant.as_str(), "filled" | "outlined" | "standard"))
        .unwrap_or_else(|| "outlined".to_string())
}

fn text_field_label_shrinks(attributes: &BTreeMap<String, Value>) -> bool {
    bool_from_attributes_any(attributes, &["shrink", "focused"])
        || string_from_attributes_any(attributes, &["value", "value_text", "defaultValue"])
            .is_some_and(|value| !value.is_empty())
}

fn append_prefixed_state_classes(node: &mut UiTemplateNode, prefix: &str, states: &[&str]) {
    for state in states {
        if bool_attribute(node, state) {
            append_class(&mut node.classes, format!("{prefix}-{state}"));
        }
    }
}
