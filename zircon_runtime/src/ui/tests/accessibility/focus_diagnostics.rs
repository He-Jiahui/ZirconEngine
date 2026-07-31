use super::*;

#[test]
fn accessibility_validation_indexes_snapshot_nodes_without_deep_cloning() {
    let source = include_str!("../../accessibility/diagnostics.rs");

    assert!(source.contains("BTreeMap<UiNodeId, usize>"));
    assert!(!source.contains("Some((node.node_id, node.clone()))"));
}

#[test]
fn focus_inside_hidden_subtree_falls_back_and_reports_excluded_focus() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-panel"))
                .with_visibility(UiVisibility::Hidden),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden-panel/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Hidden focused'")),
        )
        .unwrap();
    surface.focus.focused = Some(id(3));
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(3)).is_none());
    assert_eq!(snapshot.focused, Some(id(1)));
    assert!(snapshot.node(id(1)).unwrap().state.focused);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|node| node.state.focused)
            .count(),
        1
    );
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::ExcludedFocusedNode
            && diagnostic.node_id == Some(id(3))
    }));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::HiddenFocusable
            && diagnostic.node_id == Some(id(3))
    }));
}

#[test]
fn invalid_focus_skips_disabled_root_fallback_and_clears_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility.disabled-root"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
            enabled: false,
            ..state(false, false)
        }),
    );
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden"))
                .with_visibility(UiVisibility::Hidden)
                .with_state_flags(state(false, true))
                .with_template_metadata(metadata("TextField", "text = 'Hidden'")),
        )
        .unwrap();
    surface.focus.focused = Some(id(2));
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.focused, None);
    assert!(snapshot.nodes.iter().all(|node| !node.state.focused));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::ExcludedFocusedNode
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn disabled_nodes_are_discoverable_with_invalid_actions_filtered() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/disabled"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 20.0))
                .with_state_flags(UiStateFlags {
                    enabled: false,
                    ..state(true, true)
                })
                .with_template_metadata(metadata("Button", "text = 'Disabled'")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).unwrap();
    assert!(node.state.disabled);
    assert_eq!(node.actions, vec![UiAccessibilityAction::Focus]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DisabledAction
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn disabled_attribute_filters_invalid_actions() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/disabled-attribute"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 20.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Disabled'\ndisabled = true")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).unwrap();

    assert!(node.state.disabled);
    assert_eq!(node.actions, vec![UiAccessibilityAction::Focus]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DisabledAction
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn hidden_focusable_nodes_are_diagnosed_without_normal_inclusion() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-focusable"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 20.0))
                .with_visibility(UiVisibility::Hidden)
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    widget: UiWidgetContract {
                        tooltip: Some("Hidden input".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::HiddenFocusable
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn invalid_focus_falls_back_to_root_and_reports_diagnostic() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 20.0))
                .with_visibility(UiVisibility::Hidden)
                .with_state_flags(state(false, true))
                .with_template_metadata(metadata("TextField", "text = 'Hidden'")),
        )
        .unwrap();
    surface.focus.focused = Some(id(2));
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.focused, Some(id(1)));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::ExcludedFocusedNode
            && diagnostic.node_id == Some(id(2))
    }));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::HiddenFocusable
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn bounds_fall_back_to_layout_cache_when_arranged_tree_is_empty() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/cached"))
                .with_frame(UiFrame::new(12.0, 16.0, 48.0, 20.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Cached bounds'")),
        )
        .unwrap();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(2)).unwrap().bounds,
        Some(UiFrame::new(12.0, 16.0, 48.0, 20.0))
    );
    assert!(snapshot.diagnostics.iter().all(|diagnostic| {
        diagnostic.code != UiAccessibilityDiagnosticCode::MissingBounds
            || diagnostic.node_id != Some(id(2))
    }));
}

#[test]
fn missing_bounds_diagnostics_report_named_or_interactive_nodes() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/no-bounds"))
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("No bounds".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::MissingBounds
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn nameless_interactive_nodes_report_missing_name() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/nameless-button"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 20.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).unwrap().name.is_none());
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::MissingName
            && diagnostic.node_id == Some(id(2))
    }));
}
