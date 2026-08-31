use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityDiagnostic,
        UiAccessibilityDiagnosticCode, UiAccessibilityDiagnosticSeverity, UiAccessibilityNode,
        UiAccessibilityTreeSnapshot,
    },
    event_ui::UiNodeId,
};

#[cfg(test)]
mod indexed_focus_tests;

pub(super) fn validate_snapshot_bounded<E>(
    snapshot: &mut UiAccessibilityTreeSnapshot,
    mut observe_diagnostics: impl FnMut(usize) -> Result<(), E>,
) -> Result<(), E> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    let mut nodes = BTreeMap::new();
    for (index, node) in snapshot.nodes.iter().enumerate() {
        if !seen.insert(node.node_id) {
            diagnostics.push(diagnostic(
                UiAccessibilityDiagnosticSeverity::Error,
                UiAccessibilityDiagnosticCode::DuplicateNodeId,
                Some(node.node_id),
                "accessibility snapshot contains duplicate node id",
            ));
            observe_diagnostics(1)?;
            continue;
        }
        nodes.insert(node.node_id, index);
        observe_diagnostics(0)?;
    }

    for node in snapshot.nodes.iter() {
        let diagnostic_count = diagnostics.len();
        validate_relation(
            node,
            node.labelled_by,
            UiAccessibilityDiagnosticCode::DanglingLabel,
            &nodes,
            &mut diagnostics,
        );
        validate_relation(
            node,
            node.label_for,
            UiAccessibilityDiagnosticCode::DanglingLabel,
            &nodes,
            &mut diagnostics,
        );
        validate_description(node, &nodes, &mut diagnostics);
        validate_name(node, &mut diagnostics);
        validate_bounds(node, &mut diagnostics);
        validate_hidden_focusable(node, &mut diagnostics);
        validate_actions(node, &mut diagnostics);
        validate_relation_cycle(node, &snapshot.nodes, &nodes, &mut diagnostics);
        observe_diagnostics(diagnostics.len().saturating_sub(diagnostic_count))?;
    }

    let diagnostic_count = diagnostics.len();
    validate_focus(snapshot, &nodes, &mut diagnostics);
    observe_diagnostics(diagnostics.len().saturating_sub(diagnostic_count))?;
    snapshot.diagnostics.extend(diagnostics);
    Ok(())
}

fn validate_relation(
    owner: &UiAccessibilityNode,
    target: Option<UiNodeId>,
    code: UiAccessibilityDiagnosticCode,
    nodes: &BTreeMap<UiNodeId, usize>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) {
    if target.is_some_and(|target| !nodes.contains_key(&target)) {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Error,
            code,
            Some(owner.node_id),
            "accessibility relation points to a node outside the snapshot",
        ));
    }
}

fn validate_description(
    node: &UiAccessibilityNode,
    nodes: &BTreeMap<UiNodeId, usize>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) {
    let Some(description) = node.description.as_deref() else {
        return;
    };
    let Some(reference) = description.strip_prefix('#') else {
        return;
    };
    let Some(target) = reference.parse::<u64>().ok().map(UiNodeId::new) else {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Error,
            UiAccessibilityDiagnosticCode::DanglingDescription,
            Some(node.node_id),
            "description reference is not a valid node id",
        ));
        return;
    };
    if !nodes.contains_key(&target) {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Error,
            UiAccessibilityDiagnosticCode::DanglingDescription,
            Some(node.node_id),
            "description reference points to a node outside the snapshot",
        ));
    }
}

fn validate_bounds(node: &UiAccessibilityNode, diagnostics: &mut Vec<UiAccessibilityDiagnostic>) {
    if node.state.hidden {
        return;
    }
    if node.bounds.is_none() && (is_interactive(node) || node.name.is_some()) {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Warning,
            UiAccessibilityDiagnosticCode::MissingBounds,
            Some(node.node_id),
            "interactive or named accessibility node is missing bounds",
        ));
    }
}

fn validate_name(node: &UiAccessibilityNode, diagnostics: &mut Vec<UiAccessibilityDiagnostic>) {
    if is_interactive(node) && node.name.as_deref().is_none_or(str::is_empty) {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Warning,
            UiAccessibilityDiagnosticCode::MissingName,
            Some(node.node_id),
            "interactive or actionable accessibility node is missing an accessible name",
        ));
    }
}

fn validate_hidden_focusable(
    node: &UiAccessibilityNode,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) {
    if node.state.hidden && node.actions.contains(&UiAccessibilityAction::Focus) {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Error,
            UiAccessibilityDiagnosticCode::HiddenFocusable,
            Some(node.node_id),
            "hidden accessibility node exposes focus",
        ));
    }
}

fn validate_actions(node: &UiAccessibilityNode, diagnostics: &mut Vec<UiAccessibilityDiagnostic>) {
    for action in node.actions.iter().copied() {
        if !role_supports_action(node.role, action) {
            diagnostics.push(diagnostic(
                UiAccessibilityDiagnosticSeverity::Warning,
                UiAccessibilityDiagnosticCode::UnsupportedRoleAction,
                Some(node.node_id),
                "accessibility node exposes an action unsupported by its role",
            ));
        }
    }
    if node.state.disabled
        && node
            .actions
            .iter()
            .any(|action| *action != UiAccessibilityAction::Focus)
    {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Warning,
            UiAccessibilityDiagnosticCode::DisabledAction,
            Some(node.node_id),
            "disabled accessibility node exposes an invalid action",
        ));
    }
}

fn validate_relation_cycle(
    node: &UiAccessibilityNode,
    snapshot_nodes: &[UiAccessibilityNode],
    nodes: &BTreeMap<UiNodeId, usize>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) {
    let Some(labelled_by) = node.labelled_by else {
        return;
    };
    if nodes
        .get(&labelled_by)
        .and_then(|index| snapshot_nodes.get(*index))
        .and_then(|target| target.labelled_by)
        == Some(node.node_id)
    {
        diagnostics.push(diagnostic(
            UiAccessibilityDiagnosticSeverity::Error,
            UiAccessibilityDiagnosticCode::RelationCycle,
            Some(node.node_id),
            "accessibility label relation forms a cycle",
        ));
    }
}

fn validate_focus(
    snapshot: &mut UiAccessibilityTreeSnapshot,
    nodes: &BTreeMap<UiNodeId, usize>,
    diagnostics: &mut Vec<UiAccessibilityDiagnostic>,
) {
    for node in snapshot.nodes.iter_mut() {
        node.state.focused = false;
    }

    let Some(focused) = snapshot.focused else {
        return;
    };
    let focused_index = nodes.get(&focused).copied().filter(|index| {
        snapshot
            .nodes
            .get(*index)
            .is_some_and(|node| !node.state.hidden && !node.state.disabled)
    });
    if let Some(index) = focused_index {
        if let Some(node) = snapshot.nodes.get_mut(index) {
            node.state.focused = true;
        }
        return;
    }
    let code = if nodes.contains_key(&focused) {
        UiAccessibilityDiagnosticCode::InvalidFocus
    } else {
        UiAccessibilityDiagnosticCode::ExcludedFocusedNode
    };
    diagnostics.push(diagnostic(
        UiAccessibilityDiagnosticSeverity::Error,
        code,
        Some(focused),
        "focused runtime node is not a valid accessibility focus target",
    ));
    let fallback = snapshot.roots.iter().find_map(|root| {
        nodes.get(root).copied().and_then(|index| {
            snapshot
                .nodes
                .get(index)
                .is_some_and(|node| !node.state.hidden && !node.state.disabled)
                .then_some((*root, index))
        })
    });
    snapshot.focused = fallback.map(|(root, _)| root);
    if let Some((_, index)) = fallback {
        if let Some(node) = snapshot.nodes.get_mut(index) {
            node.state.focused = true;
        }
    }
}

fn is_interactive(node: &UiAccessibilityNode) -> bool {
    !node.actions.is_empty()
        || matches!(
            node.role,
            UiA11yRole::Button
                | UiA11yRole::Checkbox
                | UiA11yRole::Radio
                | UiA11yRole::Slider
                | UiA11yRole::TextInput
                | UiA11yRole::MenuItem
                | UiA11yRole::Tab
                | UiA11yRole::Scrollbar
        )
}

fn role_supports_action(role: UiA11yRole, action: UiAccessibilityAction) -> bool {
    match action {
        UiAccessibilityAction::Focus => true,
        UiAccessibilityAction::Activate => matches!(
            role,
            UiA11yRole::Button
                | UiA11yRole::Checkbox
                | UiA11yRole::Radio
                | UiA11yRole::MenuItem
                | UiA11yRole::Tab
                | UiA11yRole::Generic
        ),
        UiAccessibilityAction::Increment | UiAccessibilityAction::Decrement => {
            matches!(role, UiA11yRole::Slider | UiA11yRole::Scrollbar)
        }
        UiAccessibilityAction::SetValue => {
            matches!(role, UiA11yRole::TextInput | UiA11yRole::Slider)
        }
        UiAccessibilityAction::ReplaceSelectedText | UiAccessibilityAction::SetTextSelection => {
            matches!(role, UiA11yRole::TextInput)
        }
        UiAccessibilityAction::Expand | UiAccessibilityAction::Collapse => {
            matches!(
                role,
                UiA11yRole::Button | UiA11yRole::Menu | UiA11yRole::Generic
            )
        }
        UiAccessibilityAction::ScrollTo => true,
        UiAccessibilityAction::Dismiss => matches!(role, UiA11yRole::Dialog | UiA11yRole::Tooltip),
    }
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
