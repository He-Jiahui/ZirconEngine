use zircon_runtime_interface::ui::{
    binding::UiEventKind, surface::UiRenderCommandKind, tree::UiTemplateNodeMetadata,
};

use super::{bool_attribute, number_attribute, string_attribute, value_to_display_text};

pub(super) fn resolve_role(
    component: &str,
    kind: &UiRenderCommandKind,
    metadata: &UiTemplateNodeMetadata,
) -> &'static str {
    match component {
        "Button" => "Button",
        "Label" | "Text" => "Label",
        "InputField" | "SearchField" | "TextField" | "NumberField" => "InputField",
        "RangeField" | "Slider" | "RangeSlider" => "RangeField",
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner" => {
            "Progress"
        }
        "Skeleton" => "Skeleton",
        "Backdrop" => "Backdrop",
        "Paper" | "Dialog" | "AlertDialog" | "Popover" | "Popper" | "Tooltip" | "Snackbar"
        | "Menu" | "Drawer" => "Panel",
        "Toggle" | "ToggleButton" | "Switch" | "Checkbox" | "Radio" | "RadioField"
        | "SegmentedControl" | "Tab" | "Tabs" | "TabList" => "Toggle",
        "ComboBox" | "Dropdown" | "EnumField" | "FlagsField" | "SearchSelect" => "ComboBox",
        "TreeView" | "TreeRow" => "TreeView",
        "EditableTable" | "Table" => "Table",
        "AssetField" | "ObjectField" | "InstanceField" => "InputField",
        "Icon" => "Icon",
        "IconButton" => "IconButton",
        "SvgIcon" => "SvgIcon",
        _ if string_attribute(metadata, "surface_variant").is_some()
            || matches!(kind, UiRenderCommandKind::Quad) =>
        {
            "Panel"
        }
        _ if metadata.control_id.is_some() => "Mount",
        _ => "Group",
    }
}

pub(crate) fn resolve_component_role(component: &str) -> &'static str {
    match component {
        "Button" => "button",
        "Text" => "text",
        "Label" => "label",
        "Image" => "image",
        "Svg" => "svg",
        "SvgIcon" => "svg-icon",
        "Canvas" => "canvas",
        "Icon" => "icon",
        "IconButton" => "icon-button",
        "InputField" | "SearchField" | "TextField" => "input-field",
        "NumberField" => "number-field",
        "RangeField" => "range-field",
        "Slider" => "slider",
        "RangeSlider" => "range-slider",
        "Progress" => "progress",
        "ProgressBar" => "progress-bar",
        "LinearProgress" => "linear-progress",
        "CircularProgress" => "circular-progress",
        "Spinner" => "spinner",
        "Divider" | "Separator" => "divider",
        "Skeleton" => "skeleton",
        "Backdrop" => "backdrop",
        "Paper" => "paper",
        "Modal" => "modal",
        "Dialog" => "dialog",
        "AlertDialog" => "alert-dialog",
        "Popover" => "popover",
        "Popper" => "popper",
        "Tooltip" => "tooltip",
        "Snackbar" => "snackbar",
        "Menu" => "menu",
        "ContextMenu" => "context-menu",
        "ContextActionMenu" => "context-action-menu",
        "DropdownPopup" => "dropdown-popup",
        "Drawer" => "drawer",
        "Collapse" => "collapse",
        "Fade" => "fade",
        "Grow" => "grow",
        "Slide" => "slide",
        "Zoom" => "zoom",
        "Popup" => "popup",
        "Toggle" => "toggle",
        "ToggleButton" => "toggle-button",
        "Switch" => "switch",
        "Checkbox" => "checkbox",
        "SegmentedControl" => "segmented-control",
        "Tab" => "tab",
        "Tabs" => "tabs",
        "TabList" => "tab-list",
        "ComboBox" => "combo-box",
        "Dropdown" => "dropdown",
        "EnumField" => "enum-field",
        "FlagsField" => "flags-field",
        "SearchSelect" => "search-select",
        "Radio" => "radio",
        "RadioField" => "radio-field",
        "AssetField" => "asset-field",
        "ObjectField" => "object-field",
        "InstanceField" => "instance-field",
        "Foldout" => "foldout",
        "TreeView" => "tree-view",
        "TreeRow" => "tree-row",
        "EditableTable" => "editable-table",
        "Table" => "table",
        "MessageBox" => "message-box",
        _ => "",
    }
}

pub(crate) fn resolve_component_variant(metadata: &UiTemplateNodeMetadata) -> String {
    let mut variant = bool_attribute(metadata, "invisible")
        .filter(|invisible| *invisible)
        .map(|_| "invisible".to_string())
        .or_else(|| string_attribute(metadata, "mui_variant"))
        .or_else(|| string_attribute(metadata, "component_variant"))
        .or_else(|| string_attribute(metadata, "variant"))
        .unwrap_or_default();
    if let Some(animation) = string_attribute(metadata, "animation") {
        if !animation.is_empty() && !variant.split_whitespace().any(|part| part == animation) {
            if variant.is_empty() {
                variant = animation;
            } else {
                variant.push(' ');
                variant.push_str(&animation);
            }
        }
    }
    if resolve_component_role(metadata.component.as_str()) == "divider" {
        if let Some(orientation) = string_attribute(metadata, "orientation") {
            append_component_variant_token(&mut variant, &orientation);
        }
        if bool_attribute(metadata, "flexItem")
            .or_else(|| bool_attribute(metadata, "flex_item"))
            .unwrap_or(false)
        {
            append_component_variant_token(&mut variant, "flexItem");
        }
        if string_attribute(metadata, "text")
            .or_else(|| string_attribute(metadata, "label"))
            .is_some_and(|value| !value.is_empty())
        {
            append_component_variant_token(&mut variant, "withChildren");
        }
        if let Some(text_align) = string_attribute(metadata, "textAlign")
            .or_else(|| string_attribute(metadata, "text_align"))
            .filter(|value| matches!(value.as_str(), "left" | "right"))
        {
            append_component_variant_token(
                &mut variant,
                &format!("textAlign{}", pascal_case(&text_align)),
            );
        }
    }
    if resolve_component_role(metadata.component.as_str()) == "input-field" {
        if variant.is_empty() {
            variant = "outlined".to_string();
        }
        if bool_attribute(metadata, "focused").unwrap_or(false) {
            append_component_variant_token(&mut variant, "focused");
        }
        if bool_attribute(metadata, "error").unwrap_or(false)
            || string_attribute(metadata, "validation_level")
                .is_some_and(|level| matches!(level.as_str(), "error" | "danger"))
        {
            append_component_variant_token(&mut variant, "error");
        }
        if let Some(size) = string_attribute(metadata, "size") {
            append_component_variant_token(&mut variant, &size);
        }
    }
    variant
}

pub(crate) fn resolve_node_value_text(
    metadata: &UiTemplateNodeMetadata,
    display_text: &str,
    component_role: &str,
) -> String {
    if let Some(value_text) = string_attribute(metadata, "value_text") {
        return value_text;
    }
    if let Some(value) = metadata.attributes.get("value") {
        return value_to_display_text(value);
    }
    if matches!(component_role, "input-field" | "number-field") {
        let placeholder = string_attribute(metadata, "placeholder").unwrap_or_default();
        if !display_text.is_empty() && display_text != placeholder {
            return display_text.to_string();
        }
    }
    String::new()
}

pub(crate) fn resolve_node_value_number(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "value")
        .or_else(|| number_attribute(metadata, "progress"))
        .unwrap_or(0.0)
}

pub(crate) fn resolve_node_value_percent(
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
    value_number: f32,
) -> f32 {
    if let Some(value_percent) = number_attribute(metadata, "value_percent")
        .or_else(|| number_attribute(metadata, "progress_percent"))
    {
        return normalize_percent_literal(value_percent);
    }
    let value = number_attribute(metadata, "progress")
        .or_else(|| number_attribute(metadata, "value"))
        .unwrap_or(value_number);
    match (
        number_attribute(metadata, "min"),
        number_attribute(metadata, "max"),
    ) {
        (Some(min), Some(max)) if max > min => ((value - min) / (max - min)).clamp(0.0, 1.0),
        _ if is_progress_component_role(component_role) && value > 1.0 => {
            normalize_percent_literal(value)
        }
        _ => value.clamp(0.0, 1.0),
    }
}

pub(crate) fn resolve_node_popup_open(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "popup_open")
        .or_else(|| bool_attribute(metadata, "open"))
        .unwrap_or(false)
}

pub(crate) fn resolve_transition_kind(
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
) -> String {
    string_attribute(metadata, "transition_kind")
        .or_else(|| string_attribute(metadata, "transition"))
        .or_else(|| match component_role {
            "collapse" | "fade" | "grow" | "slide" | "zoom" => Some(component_role.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_transition_in(
    metadata: &UiTemplateNodeMetadata,
    has_transition: bool,
    popup_open: bool,
) -> bool {
    bool_attribute(metadata, "transition_in")
        .or_else(|| bool_attribute(metadata, "in"))
        .unwrap_or_else(|| {
            if has_transition {
                popup_open || bool_attribute(metadata, "open").unwrap_or(true)
            } else {
                true
            }
        })
}

pub(crate) fn resolve_transition_progress(
    metadata: &UiTemplateNodeMetadata,
    status: &str,
    transition_in: bool,
) -> f32 {
    number_attribute(metadata, "transition_progress")
        .or_else(|| number_attribute(metadata, "animation_progress"))
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or_else(|| match status {
            "entering" | "exiting" => 0.5,
            "entered" => 1.0,
            "exited" => 0.0,
            _ if transition_in => 1.0,
            _ => 0.0,
        })
}

pub(crate) fn default_transition_duration_ms(transition_kind: &str, transition_in: bool) -> i32 {
    match transition_kind {
        "collapse" => 300,
        "fade" | "grow" | "slide" | "zoom" if transition_in => 225,
        "fade" | "grow" | "slide" | "zoom" => 195,
        _ => 0,
    }
}

pub(crate) fn default_transition_easing(
    transition_kind: &str,
    transition_in: bool,
) -> &'static str {
    match (transition_kind, transition_in) {
        ("slide", true) => "cubic-bezier(0.0, 0, 0.2, 1)",
        ("slide", false) => "cubic-bezier(0.4, 0, 0.6, 1)",
        _ => "cubic-bezier(0.4, 0, 0.2, 1)",
    }
}

pub(crate) fn preferred_binding_id(
    metadata: &UiTemplateNodeMetadata,
    event_kind: Option<UiEventKind>,
) -> Option<String> {
    metadata
        .bindings
        .iter()
        .find(|binding| event_kind.is_none_or(|event_kind| binding.event == event_kind))
        .map(|binding| binding.id.clone())
}

pub(crate) fn resolve_edit_action_id(
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
    binding_id: &str,
) -> String {
    string_attribute(metadata, "edit_action_id")
        .or_else(|| preferred_binding_id(metadata, Some(UiEventKind::Change)))
        .or_else(|| {
            matches!(component_role, "input-field" | "number-field")
                .then(|| binding_id.to_string())
                .filter(|id| !id.is_empty())
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_commit_action_id(metadata: &UiTemplateNodeMetadata) -> String {
    string_attribute(metadata, "commit_action_id")
        .or_else(|| preferred_binding_id(metadata, Some(UiEventKind::Submit)))
        .unwrap_or_default()
}

pub(super) fn icon_button_hides_label(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "IconButton"
        && string_attribute(metadata, "icon_placement")
            .is_some_and(|placement| placement.eq_ignore_ascii_case("icon_only"))
}

fn append_component_variant_token(variant: &mut String, token: &str) {
    if token.is_empty()
        || variant
            .split_whitespace()
            .any(|part| part.eq_ignore_ascii_case(token))
    {
        return;
    }
    if !variant.is_empty() {
        variant.push(' ');
    }
    variant.push_str(token);
}

fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
}

fn normalize_percent_literal(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn is_progress_component_role(component_role: &str) -> bool {
    matches!(
        component_role,
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    )
}
