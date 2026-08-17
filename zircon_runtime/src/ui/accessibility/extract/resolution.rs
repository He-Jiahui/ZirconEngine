use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    accessibility::{
        UiAccessibilityDiagnostic, UiAccessibilityDiagnosticCode,
        UiAccessibilityDiagnosticSeverity, UiAccessibilityNode,
    },
    event_ui::UiNodeId,
    tree::UiTreeNode,
};

use crate::ui::surface::UiSurface;

use super::super::{
    budget::{AccessibilityBuildBudget, AccessibilitySnapshotBudgetError},
    name,
};
use super::{diagnostic, is_hidden, parse_node_id};

pub(super) fn preflight_node_text_sources(
    node: &UiTreeNode,
    budget: &AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    budget.preflight_value(&node.node_path, 4)?;
    let Some(metadata) = node.template_metadata.as_ref() else {
        return Ok(());
    };
    for value in [
        metadata.a11y.name.as_deref(),
        metadata.a11y.description.as_deref(),
        metadata.a11y.tooltip.as_deref(),
        metadata.widget.tooltip.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        budget.preflight_value(value, 4)?;
    }
    for key in name::TEXT_KEYS
        .iter()
        .chain(name::ALT_KEYS)
        .chain(name::TOOLTIP_KEYS)
    {
        if let Some(toml::Value::String(value)) = metadata.attributes.get(*key) {
            budget.preflight_value(value, 4)?;
        }
    }
    Ok(())
}

pub(super) fn resolve_names(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    let ids: Vec<_> = nodes.keys().copied().collect();
    for node_id in ids {
        if nodes.get(&node_id).is_some_and(|node| node.name.is_some()) {
            continue;
        }
        let resolved = labelled_by_name(surface, nodes, node_id)
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
        budget.observe_replacement(&None::<String>, &resolved, 3)?;
        if let Some(node) = nodes.get_mut(&node_id) {
            node.name = resolved;
        }
    }
    Ok(())
}

pub(super) fn resolve_descriptions(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    let ids: Vec<_> = nodes.keys().copied().collect();
    for node_id in ids {
        let Some(description) = nodes
            .get(&node_id)
            .and_then(|node| node.description.clone())
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
                description,
                "description reference is not a valid node id",
                budget,
            )?;
            continue;
        };

        if let Some(resolved) = referenced_text(surface, nodes, description_target) {
            let replacement = Some(resolved);
            budget.observe_replacement(&Some(description), &replacement, 3)?;
            if let Some(node) = nodes.get_mut(&node_id) {
                node.description = replacement;
            }
        } else if nodes.contains_key(&description_target) {
            clear_description_reference(
                nodes,
                diagnostics,
                node_id,
                description,
                "description reference target has no usable accessible text",
                budget,
            )?;
        } else {
            clear_description_reference(
                nodes,
                diagnostics,
                node_id,
                description,
                "description reference points to a node outside the snapshot",
                budget,
            )?;
        }
    }
    Ok(())
}

pub(super) fn prune_hidden_relation_targets(
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

pub(super) fn filter_children(
    surface: &UiSurface,
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    hidden_relation_targets: &BTreeSet<UiNodeId>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
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
                budget,
            )?;
        }
        if let Some(accessibility_node) = nodes.get_mut(&node.node_id) {
            accessibility_node.children = filtered;
        }
    }
    Ok(())
}

fn labelled_by_name(
    surface: &UiSurface,
    nodes: &BTreeMap<UiNodeId, UiAccessibilityNode>,
    node_id: UiNodeId,
) -> Option<String> {
    let label_id = nodes.get(&node_id)?.labelled_by?;
    referenced_text(surface, nodes, label_id)
}

fn clear_description_reference(
    nodes: &mut BTreeMap<UiNodeId, UiAccessibilityNode>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
    node_id: UiNodeId,
    previous: String,
    message: &'static str,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    budget.observe_replacement(&Some(previous), &None::<String>, 3)?;
    if let Some(node) = nodes.get_mut(&node_id) {
        node.description = None;
    }
    let diagnostic = diagnostic(
        UiAccessibilityDiagnosticSeverity::Error,
        UiAccessibilityDiagnosticCode::DanglingDescription,
        Some(node_id),
        message,
    );
    budget.observe_items(1)?;
    budget.observe_value(&diagnostic, 2)?;
    diagnostics.push(diagnostic);
    Ok(())
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

fn collect_included_children(
    surface: &UiSurface,
    node_id: UiNodeId,
    included: &BTreeSet<UiNodeId>,
    hidden_relation_targets: &BTreeSet<UiNodeId>,
    children: &mut Vec<UiNodeId>,
    budget: &mut AccessibilityBuildBudget,
) -> Result<(), AccessibilitySnapshotBudgetError> {
    budget.check_deadline()?;
    if hidden_relation_targets.contains(&node_id) {
        return Ok(());
    }
    if included.contains(&node_id) {
        budget.observe_items(1)?;
        budget.observe_value(&node_id, 3)?;
        children.push(node_id);
        return Ok(());
    }
    let Some(node) = surface.tree.nodes.get(&node_id) else {
        return Ok(());
    };
    if is_hidden(node) {
        return Ok(());
    }
    for child in node.children.iter().copied() {
        collect_included_children(
            surface,
            child,
            included,
            hidden_relation_targets,
            children,
            budget,
        )?;
    }
    Ok(())
}
