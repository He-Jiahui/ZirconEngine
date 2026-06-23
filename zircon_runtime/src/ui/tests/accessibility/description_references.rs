use super::*;

#[test]
fn description_references_resolve_to_target_text() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/help"))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata("Text", "text = 'Resolved help text'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Input".to_string()),
                        description: Some("#2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().description.as_deref(),
        Some("Resolved help text")
    );
}

#[test]
fn description_reference_to_textless_target_is_cleared_and_diagnosed() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/empty-help"))
                .with_visibility(UiVisibility::Hidden),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Input".to_string()),
                        description: Some("#2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert_eq!(snapshot.node(id(3)).unwrap().description, None);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DanglingDescription
            && diagnostic.node_id == Some(id(3))
    }));
}

#[test]
fn double_hash_description_reference_is_malformed_and_not_double_stripped() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/help"))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata("Text", "text = 'Should not resolve'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Input".to_string()),
                        description: Some("##2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert_eq!(snapshot.node(id(3)).unwrap().description, None);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DanglingDescription
            && diagnostic.node_id == Some(id(3))
    }));
}

#[test]
fn malformed_labelled_by_reports_invalid_label_reference() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("not-a-node".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::InvalidLabelReference
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn malformed_description_reference_reports_dangling_description() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Input".to_string()),
                        description: Some("#not-a-node".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.node(id(2)).unwrap().description, None);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DanglingDescription
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn dangling_description_reference_is_cleared_and_diagnosed() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Input".to_string()),
                        description: Some("#404".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.node(id(2)).unwrap().description, None);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DanglingDescription
            && diagnostic.node_id == Some(id(2))
    }));
}

#[test]
fn hidden_label_for_targets_are_not_retained_as_source_text_targets() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-control"))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata("TextField", "text = 'Hidden input'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/label"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    attributes: toml::from_str("text = 'Visible label'").unwrap(),
                    a11y: UiAccessibilityContract {
                        label_for: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert_eq!(snapshot.node(id(3)).unwrap().label_for, Some(id(2)));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DanglingLabel
            && diagnostic.node_id == Some(id(3))
    }));
}

#[test]
fn hidden_widget_label_for_targets_are_not_retained_as_source_text_targets() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-control"))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata("TextField", "text = 'Hidden input'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/widget-label"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    attributes: toml::from_str("text = 'Visible label'").unwrap(),
                    widget: UiWidgetContract {
                        label_for: Some("2".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert_eq!(snapshot.node(id(3)).unwrap().label_for, Some(id(2)));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::DanglingLabel
            && diagnostic.node_id == Some(id(3))
    }));
}

#[test]
fn two_node_labelled_by_cycles_are_diagnosed() {
    let mut surface = root_surface();
    for (node_id, target_id, path) in [(2, 3, "root/cycle-a"), (3, 2, "root/cycle-b")] {
        surface
            .tree
            .insert_child(
                id(1),
                UiTreeNode::new(id(node_id), UiNodePath::new(path))
                    .with_frame(UiFrame::new(4.0, node_id as f32 * 24.0, 80.0, 20.0))
                    .with_template_metadata(UiTemplateNodeMetadata {
                        component: "Label".to_string(),
                        a11y: UiAccessibilityContract {
                            labelled_by: Some(target_id.to_string()),
                            ..UiAccessibilityContract::default()
                        },
                        ..UiTemplateNodeMetadata::default()
                    }),
            )
            .unwrap();
    }
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::RelationCycle
            && diagnostic.node_id == Some(id(2))
    }));
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::RelationCycle
            && diagnostic.node_id == Some(id(3))
    }));
}

#[test]
fn unsupported_role_actions_are_diagnosed() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/static-text"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 20.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    attributes: toml::from_str("text = 'Static'").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Text,
                        actions: vec![UiAccessibilityAction::Activate],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract::default(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == UiAccessibilityDiagnosticCode::UnsupportedRoleAction
            && diagnostic.node_id == Some(id(2))
    }));
}
