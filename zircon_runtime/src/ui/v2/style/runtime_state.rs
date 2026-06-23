use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::component::UiComponentState;
use zircon_runtime_interface::ui::style::{
    UiPainterFamily, UiPainterResolvedState, UiPainterState, UiPainterStyleSelector,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTreeNode};
use zircon_runtime_interface::ui::v2::UiV2ArenaNode;

pub(super) fn collect_pseudo_states(node: &UiV2ArenaNode) -> Vec<String> {
    let mut states = Vec::new();
    collect_true_state_names(&node.props, &mut states);
    collect_true_state_names(&node.state, &mut states);
    append_resolved_painter_state(&node.component, &mut states);
    states.sort();
    states.dedup();
    states
}

pub(super) fn collect_runtime_pseudo_states(
    node: &UiTreeNode,
    component_state: Option<&UiComponentState>,
) -> Vec<String> {
    let mut states = Vec::new();
    let component = node
        .template_metadata
        .as_ref()
        .map(|metadata| metadata.component.as_str())
        .unwrap_or_default();
    if let Some(metadata) = node.template_metadata.as_ref() {
        collect_true_runtime_state_names(&metadata.attributes, &mut states);
    }
    if let Some(component_state) = component_state {
        collect_bool_state("hovered", component_state.flags.hovered, &mut states);
        collect_bool_state("focused", component_state.flags.focused, &mut states);
        collect_bool_state("pressed", component_state.flags.pressed, &mut states);
        collect_bool_state("checked", component_state.flags.checked, &mut states);
        collect_bool_state("disabled", component_state.flags.disabled, &mut states);
        collect_bool_state("expanded", component_state.flags.expanded, &mut states);
        collect_bool_state("popup_open", component_state.flags.popup_open, &mut states);
        collect_bool_state("selected", component_state.flags.selected, &mut states);
        collect_bool_state("dragging", component_state.flags.dragging, &mut states);
        collect_bool_state(
            "drop_hovered",
            component_state.flags.drop_hovered,
            &mut states,
        );
        collect_bool_state(
            "active_drag_target",
            component_state.flags.active_drag_target,
            &mut states,
        );
        collect_bool_state("loading", component_state.flags.loading, &mut states);
    }
    collect_bool_state("pressed", node.state_flags.pressed, &mut states);
    collect_bool_state("checked", node.state_flags.checked, &mut states);
    collect_bool_state("disabled", !node.state_flags.enabled, &mut states);
    append_resolved_painter_state(component, &mut states);
    states.sort();
    states.dedup();
    states
}

fn append_resolved_painter_state(component: &str, states: &mut Vec<String>) {
    let state = painter_state_from_selector_states(states);
    let family = painter_family_for_component(component);
    let resolved = UiPainterStyleSelector::resolved_state_for_family(state, family);
    append_resolved_state_aliases(resolved, states);
}

fn painter_state_from_selector_states(states: &[String]) -> UiPainterState {
    UiPainterState {
        hovered: has_selector_state(states, &["hover", "hovered"]),
        pressed: has_selector_state(states, &["active", "press", "pressed"]),
        focused: has_selector_state(
            states,
            &["focus", "focused", "focus-visible", "focus_visible"],
        ),
        disabled: has_selector_state(states, &["disabled"]),
        checked: has_selector_state(states, &["checked"]),
        selected: has_selector_state(states, &["selected"]),
        open: has_selector_state(states, &["open", "popup-open", "popup_open"]),
        dragging: has_selector_state(states, &["dragging"]),
        drop_hovered: has_selector_state(
            states,
            &["drop-hovered", "drop_hovered", "active_drag_target"],
        ),
        loading: has_selector_state(states, &["loading"]),
    }
}

fn has_selector_state(states: &[String], names: &[&str]) -> bool {
    states
        .iter()
        .any(|state| names.iter().any(|name| state == name))
}

fn painter_family_for_component(component: &str) -> UiPainterFamily {
    match component {
        "Button" | "MaterialButton" | "WorkbenchButton" => UiPainterFamily::Button,
        "IconButton" => UiPainterFamily::IconButton,
        "Toggle" | "Switch" => UiPainterFamily::Toggle,
        "Checkbox" | "CheckboxField" => UiPainterFamily::Checkbox,
        "Radio" | "RadioField" => UiPainterFamily::Radio,
        "Slider" | "RangeField" => UiPainterFamily::Slider,
        "Dropdown" | "ComboBox" | "EnumField" | "FlagsField" | "SearchSelect" => {
            UiPainterFamily::Dropdown
        }
        "PopupRow" | "MenuItem" | "OptionRow" => UiPainterFamily::PopupRow,
        "Alert" | "MessageBox" => UiPainterFamily::Alert,
        "Tooltip" => UiPainterFamily::Tooltip,
        "TextField" | "InputField" | "NumberField" | "ColorField" | "VectorField" => {
            UiPainterFamily::TextField
        }
        "ListRow" | "ListItem" | "PropertyRow" => UiPainterFamily::ListRow,
        "TreeRow" => UiPainterFamily::TreeRow,
        "TableRow" => UiPainterFamily::TableRow,
        "Tab" => UiPainterFamily::Tab,
        "Toast" | "Snackbar" => UiPainterFamily::Toast,
        "Chrome" | "WindowChrome" | "WindowFrame" | "DockHeader" | "StatusBar" | "ActivityRail" => {
            UiPainterFamily::Chrome
        }
        _ => UiPainterFamily::Generic,
    }
}

fn append_resolved_state_aliases(resolved: UiPainterResolvedState, states: &mut Vec<String>) {
    match resolved {
        UiPainterResolvedState::Normal => append_state(states, "resolved-normal"),
        UiPainterResolvedState::Hovered => {
            append_state(states, "resolved-hovered");
            append_state(states, "resolved-hover");
        }
        UiPainterResolvedState::Pressed => {
            append_state(states, "resolved-pressed");
            append_state(states, "resolved-active");
        }
        UiPainterResolvedState::Focused => {
            append_state(states, "resolved-focused");
            append_state(states, "resolved-focus");
        }
        UiPainterResolvedState::Disabled => append_state(states, "resolved-disabled"),
        UiPainterResolvedState::Checked => append_state(states, "resolved-checked"),
        UiPainterResolvedState::Selected => append_state(states, "resolved-selected"),
        UiPainterResolvedState::Open => {
            append_state(states, "resolved-open");
            append_state(states, "resolved-popup-open");
        }
        UiPainterResolvedState::Dragging => append_state(states, "resolved-dragging"),
        UiPainterResolvedState::DropHovered => {
            append_state(states, "resolved-drop-hovered");
            append_state(states, "resolved-drop_hovered");
        }
        UiPainterResolvedState::Loading => append_state(states, "resolved-loading"),
    }
}

fn append_state(states: &mut Vec<String>, state: &str) {
    if !states.iter().any(|value| value == state) {
        states.push(state.to_string());
    }
}

fn collect_true_state_names(values: &BTreeMap<String, Value>, states: &mut Vec<String>) {
    for (name, value) in values {
        if value.as_bool() != Some(true) {
            continue;
        }
        push_state_with_alias(name, states);
    }
}

fn collect_true_runtime_state_names(values: &BTreeMap<String, Value>, states: &mut Vec<String>) {
    for (name, value) in values {
        if value.as_bool() == Some(true) && !is_retained_runtime_state(name) {
            push_state_with_alias(name, states);
        }
    }
}

fn collect_bool_state(name: &str, enabled: bool, states: &mut Vec<String>) {
    if enabled {
        push_state_with_alias(name, states);
    }
}

fn push_state_with_alias(name: &str, states: &mut Vec<String>) {
    if !states.iter().any(|state| state == name) {
        states.push(name.to_string());
    }
    if let Some(alias) = pseudo_alias(name) {
        if !states.iter().any(|state| state == alias) {
            states.push(alias.to_string());
        }
    }
}

fn is_retained_runtime_state(name: &str) -> bool {
    matches!(
        name,
        "hover"
            | "hovered"
            | "focus"
            | "focused"
            | "active"
            | "pressed"
            | "checked"
            | "disabled"
            | "enabled"
            | "expanded"
            | "popup_open"
            | "open"
            | "selected"
            | "dragging"
            | "drop_hovered"
            | "active_drag_target"
            | "loading"
    )
}

fn pseudo_alias(name: &str) -> Option<&'static str> {
    match name {
        "hovered" => Some("hover"),
        "pressed" => Some("active"),
        "focused" => Some("focus"),
        "disabled" => Some("disabled"),
        "checked" => Some("checked"),
        "selected" => Some("selected"),
        "popup_open" => Some("open"),
        _ => None,
    }
}

pub(super) fn apply_retained_runtime_state_attributes(
    attributes: &mut BTreeMap<String, Value>,
    active_states: &[String],
) {
    let retained_keys = [
        "hover",
        "hovered",
        "focus",
        "focused",
        "active",
        "pressed",
        "checked",
        "disabled",
        "enabled",
        "expanded",
        "popup_open",
        "open",
        "selected",
        "dragging",
        "drop_hovered",
        "active_drag_target",
        "loading",
    ];
    for key in retained_keys {
        attributes.remove(key);
    }
    for state in [
        "hovered",
        "focused",
        "pressed",
        "checked",
        "disabled",
        "expanded",
        "popup_open",
        "selected",
        "dragging",
        "drop_hovered",
        "active_drag_target",
        "loading",
    ] {
        if active_states.iter().any(|active| active == state) {
            attributes.insert(state.to_string(), Value::Boolean(true));
        }
    }
}

pub(super) fn dirty_for_runtime_style_delta(
    old_attributes: &BTreeMap<String, Value>,
    new_attributes: &BTreeMap<String, Value>,
) -> UiDirtyFlags {
    let mut dirty = UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    };
    let changed_keys = old_attributes
        .keys()
        .chain(new_attributes.keys())
        .filter(|key| old_attributes.get(*key) != new_attributes.get(*key))
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in changed_keys {
        if is_retained_runtime_state(&key) {
            continue;
        }
        if is_text_affecting_style_key(&key) {
            dirty.text = true;
        } else if !is_render_only_style_key(&key) {
            dirty.style = true;
        }
    }
    dirty
}

fn is_text_affecting_style_key(key: &str) -> bool {
    matches!(
        key,
        "text"
            | "label"
            | "font"
            | "font_size"
            | "font_family"
            | "font_weight"
            | "line_height"
            | "letter_spacing"
            | "text_align"
            | "wrap"
    )
}

fn is_render_only_style_key(key: &str) -> bool {
    matches!(
        key,
        "background"
            | "background_color"
            | "fg"
            | "foreground"
            | "foreground_color"
            | "color"
            | "border"
            | "border_color"
            | "border_width"
            | "outline"
            | "outline_color"
            | "outline_width"
            | "opacity"
            | "radius"
            | "corner_radius"
            | "shadow"
            | "elevation"
            | "cursor"
            | "button_variant"
            | "button_color"
            | "button_size"
            | "button_interaction_state"
            | "icon_placement"
            | "button_icon_placement"
    )
}

pub(super) fn merge_dirty_flags_into(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}
