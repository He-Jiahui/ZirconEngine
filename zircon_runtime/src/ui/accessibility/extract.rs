use std::collections::{BTreeMap, BTreeSet};

use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiA11yState, UiAccessibilityAction, UiAccessibilityDiagnostic,
        UiAccessibilityDiagnosticCode, UiAccessibilityDiagnosticSeverity, UiAccessibilityNode,
        UiAccessibilityTreeSnapshot,
    },
    event_ui::UiNodeId,
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

use super::{
    budget::{AccessibilityBuildBudget, AccessibilitySnapshotBudgetError},
    diagnostics::validate_snapshot_bounded,
    name,
};
use state::{
    checked_state_for, disabled_state_for, expanded_state_for, pressed_state_for,
    selected_state_for, text_selection_state_for, value_state_for,
};

mod resolution;
mod state;

pub(crate) fn accessibility_snapshot(surface: &UiSurface) -> UiAccessibilityTreeSnapshot {
    let mut budget = AccessibilityBuildBudget::unbounded();
    build_accessibility_snapshot(surface, &mut budget)
        .expect("unbounded accessibility extraction cannot exhaust its budget")
}

pub(crate) fn accessibility_snapshot_bounded(
    surface: &UiSurface,
    budget: &mut AccessibilityBuildBudget,
) -> Result<UiAccessibilityTreeSnapshot, AccessibilitySnapshotBudgetError> {
    build_accessibility_snapshot(surface, budget)
}

fn build_accessibility_snapshot(
    surface: &UiSurface,
    budget: &mut AccessibilityBuildBudget,
) -> Result<UiAccessibilityTreeSnapshot, AccessibilitySnapshotBudgetError> {
    let mut nodes = BTreeMap::new();
    let mut relation_targets = BTreeSet::new();
    let mut hidden_source_relation_targets = BTreeSet::new();
    let mut hidden_relation_targets = BTreeSet::new();
    let mut diagnostics = Vec::new();
    budget.observe_value(
        &UiAccessibilityTreeSnapshot {
            tree_id: surface.tree.tree_id.clone(),
            ..UiAccessibilityTreeSnapshot::default()
        },
        0,
    )?;

    for node in surface.tree.nodes.values() {
        budget.check_deadline()?;
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
        budget.check_deadline()?;
        let is_relation_target = relation_targets.contains(&node.node_id);
        let can_retain_hidden_relation_target =
            hidden_source_relation_targets.contains(&node.node_id);
        let effectively_hidden = is_effectively_hidden(surface, node);
        if is_hidden_focusable(node, effectively_hidden) {
            let hidden_focusable = diagnostic(
                UiAccessibilityDiagnosticSeverity::Error,
                UiAccessibilityDiagnosticCode::HiddenFocusable,
                Some(node.node_id),
                "hidden focusable node is excluded from normal accessibility traversal",
            );
            budget.observe_items(1)?;
            budget.observe_value(&hidden_focusable, 2)?;
            diagnostics.push(hidden_focusable);
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
            preflight_node_text_sources(node, budget)?;
            let (accessibility_node, mut node_diagnostics) =
                build_node(surface, node, effectively_hidden);
            budget.observe_items(
                1_usize
                    .saturating_add(accessibility_node.actions.len())
                    .saturating_add(node_diagnostics.len()),
            )?;
            budget.observe_value(&accessibility_node, 2)?;
            for diagnostic in &node_diagnostics {
                budget.observe_value(diagnostic, 2)?;
            }
            diagnostics.append(&mut node_diagnostics);
            nodes.insert(node.node_id, accessibility_node);
        }
    }

    budget.check_deadline()?;
    resolve_names(surface, &mut nodes, budget)?;
    resolve_descriptions(surface, &mut nodes, &mut diagnostics, budget)?;
    prune_hidden_relation_targets(surface, &mut nodes, &mut hidden_relation_targets);
    filter_children(surface, &mut nodes, &hidden_relation_targets, budget)?;

    let mut roots = Vec::new();
    for root in surface.tree.roots.iter().copied() {
        budget.check_deadline()?;
        if nodes.contains_key(&root) && !hidden_relation_targets.contains(&root) {
            budget.observe_items(1)?;
            budget.observe_value(&root, 2)?;
            roots.push(root);
        }
    }
    let mut snapshot = UiAccessibilityTreeSnapshot {
        tree_id: surface.tree.tree_id.clone(),
        roots,
        nodes: nodes.into_values().collect(),
        focused: surface.focus.focused,
        diagnostics,
    };

    for hidden_target in hidden_relation_targets {
        budget.check_deadline()?;
        if let Some(node) = snapshot
            .nodes
            .iter_mut()
            .find(|node| node.node_id == hidden_target)
        {
            node.children.clear();
            node.actions.clear();
        }
    }

    let diagnostic_count_before_validation = snapshot.diagnostics.len();
    validate_snapshot_bounded(&mut snapshot, |count| budget.observe_items(count))?;
    for diagnostic in &snapshot.diagnostics[diagnostic_count_before_validation..] {
        budget.observe_value(diagnostic, 2)?;
    }
    budget.check_deadline()?;
    budget.validate_payload(&snapshot)?;
    Ok(snapshot)
}

fn preflight_node_text_sources(
    node: &UiTreeNode,
    budget: &AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    resolution::preflight_node_text_sources(node, budget)
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
        || name::has_own_text(metadata)
        || name::has_alt_text(metadata)
        || name::has_tooltip_text(metadata)
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

fn resolve_names(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    resolution::resolve_names(surface, nodes, budget)
}

fn resolve_descriptions(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    resolution::resolve_descriptions(surface, nodes, diagnostics, budget)
}

fn prune_hidden_relation_targets(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    hidden_relation_targets: &mut BTreeSet<UiNodeId>,
) {
    resolution::prune_hidden_relation_targets(surface, nodes, hidden_relation_targets)
}

fn filter_children(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    hidden_relation_targets: &BTreeSet<UiNodeId>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    resolution::filter_children(surface, nodes, hidden_relation_targets, budget)
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
        "InputField" | "TextField" | "LineEdit" | "TextEdit" | "SearchField" => {
            UiA11yRole::TextInput
        }
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
