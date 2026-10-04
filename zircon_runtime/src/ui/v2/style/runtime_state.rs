#[cfg(test)]
use std::cmp::Ordering;
use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::component::UiComponentState;
use zircon_runtime_interface::ui::style::{
    UiPainterFamily, UiPainterResolvedState, UiPainterState, UiPainterStyleSelector,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTreeNode};
use zircon_runtime_interface::ui::v2::UiV2ArenaNode;

const RUNTIME_PSEUDO_STATE_ALIAS_CAPACITY: usize = 2;
const RUNTIME_PSEUDO_STATE_FOCUS_VISIBLE_CAPACITY: usize = 4;
const RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY: usize = 1;
const RUNTIME_PSEUDO_STATE_RESOLVED_ALIAS_CAPACITY: usize = 2;

pub(super) fn collect_pseudo_states(node: &UiV2ArenaNode) -> Vec<String> {
    let authored_state_count = node.props.len().saturating_add(node.state.len());
    let mut states = Vec::with_capacity(pseudo_state_initial_capacity(authored_state_count));
    collect_true_state_names(&node.props, &mut states);
    collect_true_state_names(&node.state, &mut states);
    append_resolved_painter_state(&node.component, &mut states);
    states.sort();
    states.dedup();
    states
}

/// Every retained authored state contributes at least one output entry and the painter resolution
/// contributes at least one more. Keep a small extra allowance for the common alias pair while
/// avoiding an unconditional large allocation for empty nodes.
fn pseudo_state_initial_capacity(authored_state_count: usize) -> usize {
    authored_state_count.saturating_add(2)
}

fn runtime_pseudo_state_initial_capacity(
    node: &UiTreeNode,
    component_state: Option<&UiComponentState>,
) -> usize {
    let authored_capacity = node
        .template_metadata
        .as_ref()
        .map_or(0, |metadata| metadata.attributes.len());
    let component_capacity = component_state.map_or(0, |state| {
        let flags = &state.flags;
        bool_alias_capacity(flags.hovered, RUNTIME_PSEUDO_STATE_ALIAS_CAPACITY)
            .saturating_add(bool_alias_capacity(
                flags.focused,
                RUNTIME_PSEUDO_STATE_ALIAS_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.focus_visible,
                RUNTIME_PSEUDO_STATE_FOCUS_VISIBLE_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.pressed,
                RUNTIME_PSEUDO_STATE_ALIAS_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.checked,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.disabled,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.expanded,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.popup_open,
                RUNTIME_PSEUDO_STATE_ALIAS_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.selected,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.dragging,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.drop_hovered,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.active_drag_target,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
            .saturating_add(bool_alias_capacity(
                flags.loading,
                RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
            ))
    });
    let node_capacity = bool_alias_capacity(
        node.state_flags.pressed,
        RUNTIME_PSEUDO_STATE_ALIAS_CAPACITY,
    )
    .saturating_add(bool_alias_capacity(
        node.state_flags.checked,
        RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
    ))
    .saturating_add(bool_alias_capacity(
        !node.state_flags.enabled,
        RUNTIME_PSEUDO_STATE_SINGLE_ENTRY_CAPACITY,
    ));
    authored_capacity
        .saturating_add(component_capacity)
        .saturating_add(node_capacity)
        .saturating_add(RUNTIME_PSEUDO_STATE_RESOLVED_ALIAS_CAPACITY)
}

fn bool_alias_capacity(enabled: bool, capacity: usize) -> usize {
    if enabled {
        capacity
    } else {
        0
    }
}

pub(super) fn collect_runtime_pseudo_states(
    node: &UiTreeNode,
    component_state: Option<&UiComponentState>,
) -> Vec<String> {
    let mut states =
        Vec::with_capacity(runtime_pseudo_state_initial_capacity(node, component_state));
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
        collect_bool_state(
            "focus_visible",
            component_state.flags.focus_visible,
            &mut states,
        );
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
    let resolved = UiPainterStyleSelector::visual_state_for_family(state, family).primary;
    append_resolved_state_aliases(resolved, states);
}

fn painter_state_from_selector_states(states: &[String]) -> UiPainterState {
    let focus_visible =
        has_selector_state(states, &["focus-visible", "focus_visible", "focusVisible"]);
    UiPainterState {
        hovered: has_selector_state(states, &["hover", "hovered"]),
        pressed: has_selector_state(states, &["active", "press", "pressed"]),
        focused: has_selector_state(states, &["focus", "focused"]) || focus_visible,
        focus_visible,
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
    append_state(states, name);
    if matches!(name, "focus_visible" | "focus-visible" | "focusVisible") {
        append_state(states, "focused");
        append_state(states, "focus");
    }
    if let Some(alias) = pseudo_alias(name) {
        append_state(states, alias);
    }
}

fn is_retained_runtime_state(name: &str) -> bool {
    matches!(
        name,
        "hover"
            | "hovered"
            | "focus"
            | "focused"
            | "focus_visible"
            | "focus-visible"
            | "focusVisible"
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
        "focus_visible" => Some("focus-visible"),
        "focusVisible" => Some("focus-visible"),
        "disabled" => Some("disabled"),
        "checked" => Some("checked"),
        "selected" => Some("selected"),
        "popup_open" => Some("open"),
        _ => None,
    }
}

pub(super) const RETAINED_RUNTIME_STATE_CANONICAL_KEYS: &[&str] = &[
    "hovered",
    "focused",
    "focus_visible",
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
];

/// The outer option distinguishes ordinary properties from retained state keys.
/// Retained aliases and inactive keys are absent even when authored in the baseline.
pub(super) fn retained_runtime_state_attribute(
    key: &str,
    active_states: &[String],
) -> Option<Option<&'static Value>> {
    static ACTIVE_STATE: Value = Value::Boolean(true);
    if !is_retained_runtime_state(key) {
        return None;
    }
    Some(
        (RETAINED_RUNTIME_STATE_CANONICAL_KEYS.contains(&key)
            && active_states.iter().any(|state| state == key))
        .then_some(&ACTIVE_STATE),
    )
}

#[cfg(test)]
fn dirty_for_runtime_style_delta(
    old_attributes: &BTreeMap<String, Value>,
    new_attributes: &BTreeMap<String, Value>,
) -> UiDirtyFlags {
    let mut dirty = UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    };
    let mut old_entries = old_attributes.iter().peekable();
    let mut new_entries = new_attributes.iter().peekable();
    loop {
        if dirty.text && dirty.style {
            break;
        }
        match (old_entries.peek(), new_entries.peek()) {
            (Some((old_key, old_value)), Some((new_key, new_value))) => {
                match old_key.cmp(new_key) {
                    Ordering::Less => {
                        mark_runtime_style_delta_key(&mut dirty, old_key);
                        let _ = old_entries.next();
                    }
                    Ordering::Equal => {
                        if old_value != new_value {
                            mark_runtime_style_delta_key(&mut dirty, old_key);
                        }
                        let _ = old_entries.next();
                        let _ = new_entries.next();
                    }
                    Ordering::Greater => {
                        mark_runtime_style_delta_key(&mut dirty, new_key);
                        let _ = new_entries.next();
                    }
                }
            }
            (Some((old_key, _)), None) => {
                mark_runtime_style_delta_key(&mut dirty, old_key);
                let _ = old_entries.next();
            }
            (None, Some((new_key, _))) => {
                mark_runtime_style_delta_key(&mut dirty, new_key);
                let _ = new_entries.next();
            }
            (None, None) => break,
        }
    }
    dirty
}

pub(super) fn mark_runtime_style_delta_key(dirty: &mut UiDirtyFlags, key: &str) {
    if is_retained_runtime_state(key) {
        return;
    }
    if is_text_affecting_style_key(key) {
        dirty.text = true;
    } else if !is_render_only_style_key(key) {
        dirty.style = true;
    }
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

#[cfg(test)]
#[path = "tests/runtime_state.rs"]
mod tests;

#[cfg(test)]
#[path = "runtime_state/tests/capacity_tests.rs"]
mod capacity_tests;
