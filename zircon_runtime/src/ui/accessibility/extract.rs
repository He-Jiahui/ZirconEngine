use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yCheckedState, UiA11yRole, UiA11yState, UiA11yTextSelection, UiAccessibilityAction,
        UiAccessibilityDiagnostic, UiAccessibilityDiagnosticCode,
        UiAccessibilityDiagnosticSeverity, UiAccessibilityNode, UiAccessibilityTreeSnapshot,
    },
    component::{UiComponentState, UiValue},
    event_ui::UiNodeId,
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

use crate::ui::{
    surface::{ui_surface_effective_disabled, UiSurface},
    tree::UiRuntimeTreeAccessExt,
};

use super::{diagnostics::validate_snapshot, name};

pub(crate) fn accessibility_snapshot(surface: &UiSurface) -> UiAccessibilityTreeSnapshot {
    let mut nodes = BTreeMap::new();
    let mut relation_targets = BTreeSet::new();
    let mut hidden_source_relation_targets = BTreeSet::new();
    let mut hidden_relation_targets = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for node in surface.tree.nodes.values() {
        let effectively_hidden = is_effectively_hidden(surface, node);
        if include_node(surface, node, false, false, effectively_hidden) {
            collect_relation_targets(
                node,
                &mut relation_targets,
                &mut hidden_source_relation_targets,
            );
        }
    }

    for node in surface.tree.nodes.values() {
        let is_relation_target = relation_targets.contains(&node.node_id);
        let can_retain_hidden_relation_target =
            hidden_source_relation_targets.contains(&node.node_id);
        let effectively_hidden = is_effectively_hidden(surface, node);
        if is_hidden_focusable(node, effectively_hidden) {
            diagnostics.push(diagnostic(
                UiAccessibilityDiagnosticSeverity::Error,
                UiAccessibilityDiagnosticCode::HiddenFocusable,
                Some(node.node_id),
                "hidden focusable node is excluded from normal accessibility traversal",
            ));
        }
        if include_node(
            surface,
            node,
            is_relation_target,
            can_retain_hidden_relation_target,
            effectively_hidden,
        ) {
            if effectively_hidden && can_retain_hidden_relation_target {
                hidden_relation_targets.insert(node.node_id);
            }
            let (accessibility_node, mut node_diagnostics) =
                build_node(surface, node, effectively_hidden);
            diagnostics.append(&mut node_diagnostics);
            nodes.insert(node.node_id, accessibility_node);
        }
    }

    resolve_names(surface, &mut nodes);
    resolve_descriptions(surface, &mut nodes, &mut diagnostics);
    prune_hidden_relation_targets(surface, &mut nodes, &mut hidden_relation_targets);
    filter_children(surface, &mut nodes, &hidden_relation_targets);

    let roots = surface
        .tree
        .roots
        .iter()
        .copied()
        .filter(|root| nodes.contains_key(root) && !hidden_relation_targets.contains(root))
        .collect();
    let mut snapshot = UiAccessibilityTreeSnapshot {
        tree_id: surface.tree.tree_id.clone(),
        roots,
        nodes: nodes.into_values().collect(),
        focused: surface.focus.focused,
        diagnostics,
    };

    for hidden_target in hidden_relation_targets {
        if let Some(node) = snapshot
            .nodes
            .iter_mut()
            .find(|node| node.node_id == hidden_target)
        {
            node.children.clear();
            node.actions.clear();
        }
    }

    validate_snapshot(&mut snapshot);
    snapshot
}

fn collect_relation_targets(
    node: &UiTreeNode,
    targets: &mut BTreeSet<UiNodeId>,
    hidden_source_targets: &mut BTreeSet<UiNodeId>,
) {
    let Some(metadata) = node.template_metadata.as_ref() else {
        return;
    };
    if let Some(target) = metadata
        .a11y
        .labelled_by
        .as_deref()
        .and_then(resolve_reference)
    {
        targets.insert(target);
        hidden_source_targets.insert(target);
    }
    for reference in [
        metadata.a11y.label_for.as_deref(),
        metadata.widget.label_for.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(target) = resolve_reference(reference) {
            targets.insert(target);
        }
    }
    if let Some(description_target) = metadata
        .a11y
        .description
        .as_deref()
        .and_then(|description| description.strip_prefix('#'))
        .and_then(parse_node_id)
    {
        targets.insert(description_target);
        hidden_source_targets.insert(description_target);
    }
}

fn include_node(
    surface: &UiSurface,
    node: &UiTreeNode,
    is_relation_target: bool,
    can_retain_hidden_relation_target: bool,
    effectively_hidden: bool,
) -> bool {
    if effectively_hidden && !can_retain_hidden_relation_target {
        return false;
    }
    let metadata = node.template_metadata.as_ref();
    let explicit_accessibility = has_explicit_accessibility(metadata);
    if is_headless_scrollbar_widget(metadata) && !explicit_accessibility && !is_relation_target {
        return false;
    }
    surface.tree.roots.contains(&node.node_id)
        || explicit_accessibility
        || has_explicit_widget(metadata)
        || is_interactive(node)
        || name::own_text(metadata).is_some()
        || name::alt_text(metadata).is_some()
        || name::tooltip_text(metadata).is_some()
        || is_relation_target
        || surface
            .arranged_tree
            .get(node.node_id)
            .is_some_and(|arranged| arranged.supports_pointer())
}

fn has_explicit_accessibility(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| {
        metadata.a11y.role != UiA11yRole::Generic
            || metadata.a11y.name.is_some()
            || metadata.a11y.description.is_some()
            || metadata.a11y.labelled_by.is_some()
            || metadata.a11y.label_for.is_some()
            || metadata.a11y.tooltip.is_some()
            || !metadata.a11y.actions.is_empty()
    })
}

fn has_explicit_widget(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| metadata.widget != UiWidgetContract::default())
}

fn build_node(
    surface: &UiSurface,
    node: &UiTreeNode,
    effectively_hidden: bool,
) -> (UiAccessibilityNode, Vec<UiAccessibilityDiagnostic>) {
    let metadata = node.template_metadata.as_ref();
    let disabled = disabled_state_for(surface, node, metadata);
    let focused = surface.focus.focused == Some(node.node_id) && !disabled && !effectively_hidden;
    let role = role_for(node, metadata);
    let (actions, mut diagnostics) = actions_for(surface, node, metadata, disabled);
    let value = value_state_for(surface, node, metadata, role);
    let text_selection = text_selection_state_for(surface, node, metadata, role, value.as_deref());
    let labelled_by = parse_optional_reference(
        node.node_id,
        metadata.and_then(|metadata| metadata.a11y.labelled_by.as_deref()),
        UiAccessibilityDiagnosticCode::InvalidLabelReference,
        "labelled_by reference is not a valid node id",
        &mut diagnostics,
    );
    let label_for = parse_optional_reference(
        node.node_id,
        metadata
            .and_then(|metadata| metadata.a11y.label_for.as_deref())
            .or_else(|| metadata.and_then(|metadata| metadata.widget.label_for.as_deref())),
        UiAccessibilityDiagnosticCode::InvalidLabelReference,
        "label_for reference is not a valid node id",
        &mut diagnostics,
    );

    (
        UiAccessibilityNode {
            node_id: node.node_id,
            node_path: Some(node.node_path.clone()),
            role,
            name: metadata.and_then(|metadata| metadata.a11y.name.clone()),
            description: metadata.and_then(|metadata| metadata.a11y.description.clone()),
            bounds: bounds_for(surface, node),
            state: UiA11yState {
                disabled,
                hidden: effectively_hidden,
                focused,
                selected: selected_state_for(surface, node, metadata),
                expanded: expanded_state_for(surface, node, metadata),
                checked: checked_state_for(surface, node, metadata, role),
                pressed: pressed_state_for(surface, node, metadata),
                value,
                text_selection,
            },
            actions,
            children: Vec::new(),
            labelled_by,
            label_for,
            tooltip: metadata
                .and_then(|metadata| metadata.a11y.tooltip.clone())
                .or_else(|| metadata.and_then(|metadata| metadata.widget.tooltip.clone()))
                .or_else(|| name::tooltip_text(metadata)),
        },
        diagnostics,
    )
}

fn resolve_names(surface: &UiSurface, nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>) {
    let ids: Vec<_> = nodes.keys().copied().collect();
    for node_id in ids {
        let name = nodes
            .get(&node_id)
            .and_then(|node| node.name.clone())
            .or_else(|| labelled_by_name(surface, nodes, node_id))
            .or_else(|| {
                surface
                    .tree
                    .node(node_id)
                    .and_then(|node| name::own_text(node.template_metadata.as_ref()))
            })
            .or_else(|| {
                surface
                    .tree
                    .node(node_id)
                    .and_then(|node| name::alt_text(node.template_metadata.as_ref()))
            })
            .or_else(|| nodes.get(&node_id).and_then(|node| node.tooltip.clone()));
        if let Some(node) = nodes.get_mut(&node_id) {
            node.name = name;
        }
    }
}

fn labelled_by_name(
    surface: &UiSurface,
    nodes: &BTreeMap<UiNodeId, UiAccessibilityNode>,
    node_id: UiNodeId,
) -> Option<String> {
    let label_id = nodes.get(&node_id)?.labelled_by?;
    referenced_text(surface, nodes, label_id)
}

fn resolve_descriptions(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) {
    let ids: Vec<_> = nodes.keys().copied().collect();
    for node_id in ids {
        let Some(description) = nodes
            .get(&node_id)
            .and_then(|node| node.description.as_deref())
        else {
            continue;
        };

        let Some(reference) = description.strip_prefix('#') else {
            continue;
        };

        let Some(description_target) = parse_node_id(reference) else {
            clear_description_reference(
                nodes,
                diagnostics,
                node_id,
                "description reference is not a valid node id",
            );
            continue;
        };

        if let Some(description) = referenced_text(surface, nodes, description_target) {
            if let Some(node) = nodes.get_mut(&node_id) {
                node.description = Some(description);
            }
        } else if nodes.contains_key(&description_target) {
            clear_description_reference(
                nodes,
                diagnostics,
                node_id,
                "description reference target has no usable accessible text",
            );
        } else {
            clear_description_reference(
                nodes,
                diagnostics,
                node_id,
                "description reference points to a node outside the snapshot",
            );
        }
    }
}

fn clear_description_reference(
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
    node_id: UiNodeId,
    message: &'static str,
) {
    if let Some(node) = nodes.get_mut(&node_id) {
        node.description = None;
    }
    diagnostics.push(diagnostic(
        UiAccessibilityDiagnosticSeverity::Error,
        UiAccessibilityDiagnosticCode::DanglingDescription,
        Some(node_id),
        message,
    ));
}

fn prune_hidden_relation_targets(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    hidden_relation_targets: &mut BTreeSet<UiNodeId>,
) {
    let unusable_targets: Vec<_> = hidden_relation_targets
        .iter()
        .copied()
        .filter(|target| referenced_text(surface, nodes, *target).is_none())
        .collect();
    for target in unusable_targets {
        hidden_relation_targets.remove(&target);
        nodes.remove(&target);
    }
}

fn referenced_text(
    surface: &UiSurface,
    nodes: &BTreeMap<UiNodeId, UiAccessibilityNode>,
    target_id: UiNodeId,
) -> Option<String> {
    if !nodes.contains_key(&target_id) {
        return None;
    }
    surface
        .tree
        .node(target_id)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| {
            metadata
                .a11y
                .name
                .clone()
                .or_else(|| name::own_text(Some(metadata)))
                .or_else(|| name::alt_text(Some(metadata)))
                .or_else(|| metadata.a11y.tooltip.clone())
                .or_else(|| metadata.widget.tooltip.clone())
                .or_else(|| name::tooltip_text(Some(metadata)))
        })
}

fn filter_children(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    hidden_relation_targets: &BTreeSet<UiNodeId>,
) {
    let included: BTreeSet<_> = nodes.keys().copied().collect();
    for node in surface.tree.nodes.values() {
        let mut filtered = Vec::new();
        for child in node.children.iter().copied() {
            collect_included_children(
                surface,
                child,
                &included,
                hidden_relation_targets,
                &mut filtered,
            );
        }
        if let Some(accessibility_node) = nodes.get_mut(&node.node_id) {
            accessibility_node.children = filtered;
        }
    }
}

fn collect_included_children(
    surface: &UiSurface,
    node_id: UiNodeId,
    included: &BTreeSet<UiNodeId>,
    hidden_relation_targets: &BTreeSet<UiNodeId>,
    children: &mut Vec<UiNodeId>,
) {
    if hidden_relation_targets.contains(&node_id) {
        return;
    }
    if included.contains(&node_id) {
        children.push(node_id);
        return;
    }

    let Some(node) = surface.tree.nodes.get(&node_id) else {
        return;
    };
    if is_hidden(node) {
        return;
    }
    for child in node.children.iter().copied() {
        collect_included_children(surface, child, included, hidden_relation_targets, children);
    }
}

fn role_for(node: &UiTreeNode, metadata: Option<&UiTemplateNodeMetadata>) -> UiA11yRole {
    metadata
        .filter(|metadata| metadata.a11y.role != UiA11yRole::Generic)
        .map(|metadata| metadata.a11y.role)
        .unwrap_or_else(|| inferred_role(node, metadata))
}

fn inferred_role(node: &UiTreeNode, metadata: Option<&UiTemplateNodeMetadata>) -> UiA11yRole {
    if let Some(role) = metadata.and_then(role_for_widget_behavior) {
        return role;
    }

    let component = metadata.map_or("", |metadata| metadata.component.as_str());
    match component {
        "Button" | "IconButton" | "ToggleButton" => UiA11yRole::Button,
        "Checkbox" | "Switch" => UiA11yRole::Checkbox,
        "RadioGroup" | "ButtonGroup" => UiA11yRole::RadioGroup,
        "Radio" | "RadioButton" => UiA11yRole::Radio,
        "Slider" | "RangeField" => UiA11yRole::Slider,
        "InputField" | "TextField" | "LineEdit" | "TextEdit" => UiA11yRole::TextInput,
        "Label" | "Text" => UiA11yRole::Text,
        "Image" | "Icon" => UiA11yRole::Image,
        "List" => UiA11yRole::List,
        "ListItem" | "ListRow" => UiA11yRole::ListItem,
        "Menu" => UiA11yRole::Menu,
        "MenuItem" => UiA11yRole::MenuItem,
        "Tab" => UiA11yRole::Tab,
        "TabList" => UiA11yRole::TabList,
        "Dialog" => UiA11yRole::Dialog,
        "Tooltip" => UiA11yRole::Tooltip,
        _ if is_interactive(node) => UiA11yRole::Button,
        _ => UiA11yRole::Generic,
    }
}

fn role_for_widget_behavior(metadata: &UiTemplateNodeMetadata) -> Option<UiA11yRole> {
    match widget_behavior(metadata) {
        UiWidgetBehavior::Button => Some(UiA11yRole::Button),
        UiWidgetBehavior::MenuItem => Some(UiA11yRole::MenuItem),
        UiWidgetBehavior::Toggle => Some(UiA11yRole::Checkbox),
        UiWidgetBehavior::RadioGroup => Some(UiA11yRole::RadioGroup),
        UiWidgetBehavior::Radio => Some(UiA11yRole::Radio),
        UiWidgetBehavior::Disclosure | UiWidgetBehavior::Popup => Some(UiA11yRole::Button),
        UiWidgetBehavior::Range => Some(UiA11yRole::Slider),
        UiWidgetBehavior::TextInput => Some(UiA11yRole::TextInput),
        UiWidgetBehavior::Auto
        | UiWidgetBehavior::Passive
        | UiWidgetBehavior::Scrollbar
        | UiWidgetBehavior::ScrollbarThumb => None,
    }
}

fn expanded_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<bool> {
    let metadata = metadata?;
    match widget_behavior(metadata) {
        UiWidgetBehavior::Disclosure => {
            let property = metadata
                .widget
                .open_property
                .as_deref()
                .unwrap_or("expanded");
            Some(open_state_for(
                surface,
                node,
                metadata,
                property,
                "expanded",
                &["expanded"],
                default_expanded_state(metadata),
            ))
        }
        UiWidgetBehavior::Popup => {
            let property = metadata
                .widget
                .open_property
                .as_deref()
                .unwrap_or("popup_open");
            Some(open_state_for(
                surface,
                node,
                metadata,
                property,
                "popup_open",
                &["popup_open", "open"],
                false,
            ))
        }
        _ => bool_attribute_value(&metadata.attributes, "expanded"),
    }
}

fn open_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: &UiTemplateNodeMetadata,
    property: &str,
    canonical_property: &str,
    legacy_properties: &[&str],
    default_value: bool,
) -> bool {
    let component_state = surface.component_states.get(node.node_id);
    bool_attribute_value(&metadata.attributes, property)
        .or_else(|| component_state.and_then(|state| bool_component_state_value(state, property)))
        .or_else(|| {
            legacy_properties
                .iter()
                .copied()
                .filter(|legacy_property| *legacy_property != property)
                .find_map(|legacy_property| {
                    bool_attribute_value(&metadata.attributes, legacy_property)
                })
        })
        .or_else(|| {
            component_state.and_then(|state| {
                legacy_properties
                    .iter()
                    .copied()
                    .filter(|legacy_property| *legacy_property != property)
                    .find_map(|legacy_property| bool_component_state_value(state, legacy_property))
            })
        })
        .or_else(|| {
            component_state.and_then(|state| open_component_state_flag(state, canonical_property))
        })
        .unwrap_or(default_value)
}

fn open_component_state_flag(state: &UiComponentState, canonical_property: &str) -> Option<bool> {
    match canonical_property {
        "expanded" => state.flags.expanded.then_some(true),
        "popup_open" => state.flags.popup_open.then_some(true),
        _ => None,
    }
}

fn default_expanded_state(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Group" | "InspectorSection" | "TreeView"
    )
}

fn disabled_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    ui_surface_effective_disabled(surface, node.node_id, node, metadata)
}

fn selected_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata
        .and_then(|metadata| bool_attribute_value(&metadata.attributes, "selected"))
        .or_else(|| {
            surface
                .component_states
                .get(node.node_id)
                .and_then(|state| {
                    bool_component_state_value(state, "selected")
                        .or_else(|| state.flags.selected.then_some(true))
                })
        })
        .unwrap_or(false)
}

fn pressed_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<bool> {
    let pressed = surface
        .component_states
        .get(node.node_id)
        .is_some_and(|state| {
            bool_component_state_value(state, "pressed")
                .or_else(|| bool_component_state_value(state, "active"))
                .or_else(|| state.flags.pressed.then_some(true))
                .unwrap_or(false)
        })
        || metadata.is_some_and(|metadata| {
            bool_attribute_value(&metadata.attributes, "pressed") == Some(true)
                || bool_attribute_value(&metadata.attributes, "active") == Some(true)
        })
        || node.state_flags.pressed;
    pressed.then_some(true)
}

fn checked_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    role: UiA11yRole,
) -> Option<UiA11yCheckedState> {
    let checked = metadata
        .and_then(checked_attribute_value_for)
        .or_else(|| checked_component_state_value_for(surface, node, metadata))
        .or_else(|| metadata.and_then(|metadata| metadata.widget.checked))
        .or_else(|| {
            if node.state_flags.checked {
                Some(true)
            } else if matches!(role, UiA11yRole::Checkbox | UiA11yRole::Radio) {
                Some(false)
            } else {
                None
            }
        })?;
    Some(if checked {
        UiA11yCheckedState::True
    } else {
        UiA11yCheckedState::False
    })
}

fn checked_attribute_value_for(metadata: &UiTemplateNodeMetadata) -> Option<bool> {
    let property = metadata
        .widget
        .checked_property
        .as_deref()
        .unwrap_or("checked");
    bool_attribute_value(&metadata.attributes, property)
}

fn checked_component_state_value_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<bool> {
    let state = surface.component_states.get(node.node_id)?;
    let property = metadata
        .and_then(|metadata| metadata.widget.checked_property.as_deref())
        .unwrap_or("checked");
    if property == "checked" {
        return bool_component_state_value(state, property)
            .or_else(|| state.flags.checked.then_some(true));
    }
    bool_component_state_value(state, property)
}

fn value_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    role: UiA11yRole,
) -> Option<String> {
    let metadata = metadata?;
    value_attribute_text(metadata, role)
        .or_else(|| component_state_value_text(surface, node.node_id, metadata, role))
        .or_else(|| {
            metadata
                .widget
                .value
                .as_ref()
                .map(|value| value.display_text())
        })
}

fn value_attribute_text(metadata: &UiTemplateNodeMetadata, role: UiA11yRole) -> Option<String> {
    if let Some(property) = metadata.widget.value_property.as_deref() {
        return metadata
            .attributes
            .get(property)
            .and_then(attribute_display_text);
    }
    metadata
        .attributes
        .get("value")
        .and_then(attribute_display_text)
        .or_else(|| {
            matches!(role, UiA11yRole::TextInput)
                .then(|| {
                    metadata
                        .attributes
                        .get("text")
                        .and_then(attribute_display_text)
                })
                .flatten()
        })
}

fn text_selection_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    role: UiA11yRole,
    value_text: Option<&str>,
) -> Option<UiA11yTextSelection> {
    if role != UiA11yRole::TextInput {
        return None;
    }
    let metadata = metadata?;
    let value_text = value_text.unwrap_or_default();
    let component_state = surface.component_states.get(node.node_id);
    let caret = usize_attribute_or_component_state_value(metadata, component_state, "caret_offset")
        .unwrap_or_else(|| value_text.len());
    let anchor =
        usize_attribute_or_component_state_value(metadata, component_state, "selection_anchor")
            .unwrap_or(caret);
    let focus =
        usize_attribute_or_component_state_value(metadata, component_state, "selection_focus")
            .unwrap_or(caret);

    Some(UiA11yTextSelection {
        caret: clamp_text_byte_offset(value_text, caret),
        anchor: clamp_text_byte_offset(value_text, anchor),
        focus: clamp_text_byte_offset(value_text, focus),
    })
}

fn usize_attribute_or_component_state_value(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
    property: &str,
) -> Option<usize> {
    usize_attribute_value(&metadata.attributes, property)
        .or_else(|| component_state.and_then(|state| usize_component_state_value(state, property)))
}

fn usize_attribute_value(
    attributes: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Option<usize> {
    attributes
        .get(property)
        .and_then(toml::Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
}

fn usize_component_state_value(state: &UiComponentState, property: &str) -> Option<usize> {
    match state.value(property) {
        Some(UiValue::Int(value)) => usize::try_from(*value).ok(),
        Some(UiValue::Float(value))
            if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 =>
        {
            Some(*value as usize)
        }
        Some(UiValue::String(value)) => value.parse::<usize>().ok(),
        _ => None,
    }
}

fn clamp_text_byte_offset(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn bool_component_state_value(state: &UiComponentState, property: &str) -> Option<bool> {
    match state.value(property) {
        Some(UiValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn component_state_value_text(
    surface: &UiSurface,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    role: UiA11yRole,
) -> Option<String> {
    let state = surface.component_states.get(node_id)?;
    if let Some(property) = metadata.widget.value_property.as_deref() {
        return state.value(property).map(|value| value.display_text());
    }
    state
        .value("value")
        .map(|value| value.display_text())
        .or_else(|| {
            matches!(role, UiA11yRole::TextInput)
                .then(|| state.value("text").map(|value| value.display_text()))
                .flatten()
        })
}

fn actions_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    disabled: bool,
) -> (Vec<UiAccessibilityAction>, Vec<UiAccessibilityDiagnostic>) {
    let role = role_for(node, metadata);
    let mut actions = metadata
        .map(|metadata| metadata.a11y.actions.clone())
        .unwrap_or_default();
    let headless_scrollbar = is_headless_scrollbar_widget(metadata);
    if actions.is_empty() && !headless_scrollbar {
        if let Some(metadata) = metadata {
            actions.extend(default_actions_for_widget_behavior(
                surface, node, metadata, role,
            ));
        }
    }
    if actions.is_empty()
        && !headless_scrollbar
        && (node.state_flags.clickable || node.state_flags.pressed)
    {
        actions.push(UiAccessibilityAction::Activate);
    }
    if node.state_flags.focusable || node.focus.focusable {
        actions.push(UiAccessibilityAction::Focus);
    }
    if node.container.is_scrollable() {
        actions.push(UiAccessibilityAction::ScrollTo);
    }
    if role == UiA11yRole::Tooltip {
        actions.push(UiAccessibilityAction::Dismiss);
    }
    actions.sort();
    actions.dedup();
    let had_disabled_invalid_action = disabled
        && actions
            .iter()
            .any(|action| *action != UiAccessibilityAction::Focus);
    if disabled {
        actions.retain(|action| *action == UiAccessibilityAction::Focus);
    }
    let diagnostics = if had_disabled_invalid_action {
        vec![disabled_action_diagnostic(node.node_id)]
    } else {
        Vec::new()
    };
    (actions, diagnostics)
}

fn default_actions_for_widget_behavior(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: &UiTemplateNodeMetadata,
    role: UiA11yRole,
) -> Vec<UiAccessibilityAction> {
    match widget_behavior(metadata) {
        UiWidgetBehavior::Button
        | UiWidgetBehavior::MenuItem
        | UiWidgetBehavior::Toggle
        | UiWidgetBehavior::Radio => vec![UiAccessibilityAction::Activate],
        UiWidgetBehavior::Popup if role == UiA11yRole::Dialog => {
            if expanded_state_for(surface, node, Some(metadata)).unwrap_or(false) {
                vec![UiAccessibilityAction::Dismiss]
            } else {
                Vec::new()
            }
        }
        UiWidgetBehavior::Popup if role == UiA11yRole::Menu => {
            if expanded_state_for(surface, node, Some(metadata)).unwrap_or(false) {
                vec![UiAccessibilityAction::Collapse]
            } else {
                vec![UiAccessibilityAction::Expand]
            }
        }
        UiWidgetBehavior::Disclosure | UiWidgetBehavior::Popup => {
            let mut actions = vec![UiAccessibilityAction::Activate];
            actions.push(
                if expanded_state_for(surface, node, Some(metadata)).unwrap_or(false) {
                    UiAccessibilityAction::Collapse
                } else {
                    UiAccessibilityAction::Expand
                },
            );
            actions
        }
        UiWidgetBehavior::Range => vec![
            UiAccessibilityAction::Increment,
            UiAccessibilityAction::Decrement,
            UiAccessibilityAction::SetValue,
        ],
        UiWidgetBehavior::TextInput => vec![
            UiAccessibilityAction::SetValue,
            UiAccessibilityAction::ReplaceSelectedText,
            UiAccessibilityAction::SetTextSelection,
        ],
        UiWidgetBehavior::Auto
        | UiWidgetBehavior::Passive
        | UiWidgetBehavior::RadioGroup
        | UiWidgetBehavior::Scrollbar
        | UiWidgetBehavior::ScrollbarThumb => Vec::new(),
    }
}

fn widget_behavior(metadata: &UiTemplateNodeMetadata) -> UiWidgetBehavior {
    metadata.widget.resolved_behavior(&metadata.component)
}

fn is_headless_scrollbar_widget(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| {
        matches!(
            widget_behavior(metadata),
            UiWidgetBehavior::Scrollbar | UiWidgetBehavior::ScrollbarThumb
        )
    })
}

fn bool_attribute_value(
    attributes: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Option<bool> {
    attributes.get(property).and_then(toml::Value::as_bool)
}

fn attribute_display_text(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) if value.is_finite() => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        toml::Value::Datetime(value) => Some(value.to_string()),
        _ => None,
    }
}

fn bounds_for(surface: &UiSurface, node: &UiTreeNode) -> Option<UiFrame> {
    surface
        .arranged_tree
        .get(node.node_id)
        .map(|arranged| arranged.frame)
        .filter(valid_bounds)
        .or_else(|| Some(node.layout_cache.frame).filter(valid_bounds))
}

fn is_interactive(node: &UiTreeNode) -> bool {
    node.state_flags.clickable
        || node.state_flags.hoverable
        || node.state_flags.focusable
        || node.focus.focusable
}

fn is_hidden(node: &UiTreeNode) -> bool {
    !node.is_render_visible()
}

fn is_effectively_hidden(surface: &UiSurface, node: &UiTreeNode) -> bool {
    if is_hidden(node) {
        return true;
    }
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = surface.tree.nodes.get(&parent_id) else {
            return false;
        };
        if is_hidden(parent_node) {
            return true;
        }
        parent = parent_node.parent;
    }
    false
}

fn is_hidden_focusable(node: &UiTreeNode, effectively_hidden: bool) -> bool {
    effectively_hidden && (node.state_flags.focusable || node.focus.focusable)
}

fn valid_bounds(frame: &UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn resolve_reference(reference: &str) -> Option<UiNodeId> {
    parse_node_id(reference.strip_prefix('#').unwrap_or(reference))
}

fn parse_node_id(reference: &str) -> Option<UiNodeId> {
    reference.parse::<u64>().ok().map(UiNodeId::new)
}

fn parse_optional_reference(
    owner: UiNodeId,
    reference: Option<&str>,
    code: UiAccessibilityDiagnosticCode,
    message: &'static str,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) -> Option<UiNodeId> {
    let reference = reference?;
    let resolved = resolve_reference(reference);
    if resolved.is_none() {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Error,
            code,
            Some(owner),
            message,
        ));
    }
    resolved
}

fn disabled_action_diagnostic(node_id: UiNodeId) -> UiAccessibilityDiagnostic {
    diagnostic(
        UiAccessibilityDiagnosticSeverity::Warning,
        UiAccessibilityDiagnosticCode::DisabledAction,
        Some(node_id),
        "disabled accessibility node had invalid actions filtered",
    )
}

fn diagnostic(
    severity: UiAccessibilityDiagnosticSeverity,
    code: UiAccessibilityDiagnosticCode,
    node_id: Option<UiNodeId>,
    message: impl Into<String>,
) -> UiAccessibilityDiagnostic {
    UiAccessibilityDiagnostic {
        severity,
        code,
        node_id,
        message: message.into(),
    }
}
