use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, bool_from_attributes_any, pascal_case,
    string_attribute_any, string_from_attributes_any,
};

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "Autocomplete" => append_autocomplete_classes(node, prefix),
        _ => return false,
    }
    true
}

pub(super) fn append_slot_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) -> bool {
    match (owner_component, slot_name) {
        ("Autocomplete", "inputRoot") => {
            append_autocomplete_input_root_slot_classes(child, owner_attributes)
        }
        ("Autocomplete", "input") => {
            append_autocomplete_input_slot_classes(child, owner_attributes)
        }
        ("Autocomplete", "tag") => append_autocomplete_tag_slot_classes(child, owner_attributes),
        ("Autocomplete", "popupIndicator") => {
            append_autocomplete_popup_indicator_slot_classes(child, owner_attributes)
        }
        ("Autocomplete", "popper") => {
            append_autocomplete_popper_slot_classes(child, owner_attributes)
        }
        (
            "Autocomplete",
            "endAdornment" | "clearIndicator" | "paper" | "listbox" | "loading" | "noOptions"
            | "option" | "groupLabel" | "groupUl",
        ) => {
            append_class(&mut child.classes, format!("MuiAutocomplete-{slot_name}"));
            if slot_name == "option" {
                append_autocomplete_option_slot_classes(child, owner_attributes);
            }
        }
        _ => return false,
    }
    true
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(component, "Autocomplete")
}

fn append_autocomplete_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["popup_open", "popupOpen", "open"]) {
        append_class(&mut node.classes, format!("{prefix}-expanded"));
    }
    if bool_attribute(node, "focused") {
        append_class(&mut node.classes, format!("{prefix}-focused"));
    }
    if bool_attribute_any(node, &["fullWidth", "full_width"]) {
        append_class(&mut node.classes, format!("{prefix}-fullWidth"));
    }
    if autocomplete_has_clear_icon(node) {
        append_class(&mut node.classes, format!("{prefix}-hasClearIcon"));
    }
    if autocomplete_has_popup_icon(node) {
        append_class(&mut node.classes, format!("{prefix}-hasPopupIcon"));
    }
}

fn append_autocomplete_input_root_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiAutocomplete-inputRoot".to_string());
    if autocomplete_owner_has_clear_icon(owner_attributes) {
        append_class(
            &mut child.classes,
            "MuiAutocomplete-hasClearIcon".to_string(),
        );
    }
    if autocomplete_owner_has_popup_icon(owner_attributes) {
        append_class(
            &mut child.classes,
            "MuiAutocomplete-hasPopupIcon".to_string(),
        );
    }
}

fn append_autocomplete_input_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiAutocomplete-input".to_string());
    if bool_from_attributes_any(owner_attributes, &["inputFocused", "focused"]) {
        append_class(
            &mut child.classes,
            "MuiAutocomplete-inputFocused".to_string(),
        );
    }
}

fn append_autocomplete_tag_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiAutocomplete-tag".to_string());
    let size = string_from_attributes_any(owner_attributes, &["size", "mui_size"])
        .unwrap_or_else(|| "medium".to_string());
    append_class(
        &mut child.classes,
        format!("MuiAutocomplete-tagSize{}", pascal_case(&size)),
    );
}

fn append_autocomplete_popup_indicator_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(
        &mut child.classes,
        "MuiAutocomplete-popupIndicator".to_string(),
    );
    if bool_from_attributes_any(owner_attributes, &["popup_open", "popupOpen", "open"]) {
        append_class(
            &mut child.classes,
            "MuiAutocomplete-popupIndicatorOpen".to_string(),
        );
    }
}

fn append_autocomplete_popper_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiAutocomplete-popper".to_string());
    if bool_from_attributes_any(owner_attributes, &["disablePortal", "disable_portal"]) {
        append_class(
            &mut child.classes,
            "MuiAutocomplete-popperDisablePortal".to_string(),
        );
    }
}

fn append_autocomplete_option_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if array_attribute_any_non_empty_from_attributes(
        owner_attributes,
        &["focused_options", "focusedOptions"],
    ) || bool_attribute(child, "focused")
    {
        append_class(&mut child.classes, "MuiAutocomplete-focused".to_string());
    }
    if bool_attribute_any(child, &["focusVisible", "focus_visible"]) {
        append_class(
            &mut child.classes,
            "MuiAutocomplete-focusVisible".to_string(),
        );
    }
}

fn autocomplete_has_clear_icon(node: &UiTemplateNode) -> bool {
    !bool_attribute_any(node, &["disableClearable", "disable_clearable"])
        && !bool_attribute(node, "disabled")
        && !bool_attribute_any(
            node,
            &["readOnly", "read_only", "inputReadOnly", "input_read_only"],
        )
        && autocomplete_has_value(node)
}

fn autocomplete_has_popup_icon(node: &UiTemplateNode) -> bool {
    match string_attribute_any(node, &["forcePopupIcon", "force_popup_icon"]).as_deref() {
        Some("false") => false,
        Some("true") => true,
        _ => !bool_attribute_any(node, &["freeSolo", "free_solo"]),
    }
}

fn autocomplete_owner_has_clear_icon(owner_attributes: &BTreeMap<String, Value>) -> bool {
    !bool_from_attributes_any(owner_attributes, &["disableClearable", "disable_clearable"])
        && !bool_from_attributes_any(owner_attributes, &["disabled"])
        && !bool_from_attributes_any(
            owner_attributes,
            &["readOnly", "read_only", "inputReadOnly", "input_read_only"],
        )
        && autocomplete_owner_has_value(owner_attributes)
}

fn autocomplete_owner_has_popup_icon(owner_attributes: &BTreeMap<String, Value>) -> bool {
    match string_from_attributes_any(owner_attributes, &["forcePopupIcon", "force_popup_icon"])
        .as_deref()
    {
        Some("false") => false,
        Some("true") => true,
        _ => !bool_from_attributes_any(owner_attributes, &["freeSolo", "free_solo"]),
    }
}

fn autocomplete_has_value(node: &UiTemplateNode) -> bool {
    string_attribute_any(node, &["value", "value_text"])
        .or_else(|| string_attribute_any(node, &["query", "inputValue"]))
        .is_some()
        || array_attribute_any_non_empty(node, &["selected_options", "selectedOptions"])
}

fn autocomplete_owner_has_value(owner_attributes: &BTreeMap<String, Value>) -> bool {
    string_from_attributes_any(
        owner_attributes,
        &["value", "value_text", "query", "inputValue"],
    )
    .is_some()
        || array_attribute_any_non_empty_from_attributes(
            owner_attributes,
            &["selected_options", "selectedOptions"],
        )
}

fn array_attribute_any_non_empty(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(|values| !values.is_empty())
    })
}

fn array_attribute_any_non_empty_from_attributes(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(|values| !values.is_empty())
    })
}
