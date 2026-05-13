use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityContract,
        UiAccessibilityDiagnosticCode,
    },
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
    widget::UiWidgetContract,
};

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

fn metadata(component: &str, attributes: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes: toml::from_str(attributes).unwrap(),
        ..UiTemplateNodeMetadata::default()
    }
}

fn state(clickable: bool, focusable: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable,
        hoverable: clickable,
        focusable,
        ..UiStateFlags::default()
    }
}

fn root_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0)),
    );
    surface
}

fn dispatch_accessibility(
    surface: &mut UiSurface,
    target: UiNodeId,
    action: UiAccessibilityAction,
) -> UiInputDispatchResult {
    dispatch_accessibility_with_value(surface, target, action, None, None)
}

fn dispatch_accessibility_with_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    action: UiAccessibilityAction,
    value: Option<&str>,
    numeric_value: Option<f64>,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target,
                    action,
                    value: value.map(str::to_string),
                    numeric_value,
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap()
}

fn has_note(result: &UiInputDispatchResult, needle: &str) -> bool {
    result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.contains(needle))
}

#[test]
fn extraction_includes_widget_only_contract_nodes() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/value"))
                .with_frame(UiFrame::new(8.0, 8.0, 96.0, 20.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ValueChip".to_string(),
                    widget: UiWidgetContract {
                        value: Some(UiValue::String("42".to_string())),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("widget-only node included");
    assert_eq!(node.state.value.as_deref(), Some("42"));
}

#[test]
fn extraction_includes_interactive_text_alt_and_explicit_nodes() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Run'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/image"))
                .with_frame(UiFrame::new(10.0, 40.0, 32.0, 32.0))
                .with_template_metadata(metadata("Image", "alt = 'Preview thumbnail'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(4), UiNodePath::new("root/panel"))
                .with_frame(UiFrame::new(100.0, 10.0, 50.0, 50.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Panel,
                        name: Some("Details".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.node(id(2)).unwrap().name.as_deref(), Some("Run"));
    assert_eq!(snapshot.node(id(2)).unwrap().role, UiA11yRole::Button);
    assert!(snapshot
        .node(id(2))
        .unwrap()
        .actions
        .contains(&UiAccessibilityAction::Activate));
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Preview thumbnail")
    );
    assert_eq!(snapshot.node(id(4)).unwrap().role, UiA11yRole::Panel);
    assert_eq!(
        snapshot.node(id(1)).unwrap().children,
        vec![id(2), id(3), id(4)]
    );
}

#[test]
fn name_priority_uses_explicit_labelled_by_text_alt_then_tooltip() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/label"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 16.0))
                .with_template_metadata(metadata("Label", "text = 'Label source'")),
        )
        .unwrap();
    for (node_id, path, a11y, attributes) in [
        (
            3,
            "root/explicit",
            UiAccessibilityContract {
                name: Some("Explicit".to_string()),
                ..UiAccessibilityContract::default()
            },
            "text = 'Ignored'",
        ),
        (
            4,
            "root/labelled",
            UiAccessibilityContract {
                labelled_by: Some("2".to_string()),
                ..UiAccessibilityContract::default()
            },
            "text = 'Ignored own text'",
        ),
        (
            5,
            "root/text",
            UiAccessibilityContract::default(),
            "text = 'Own text'",
        ),
        (
            6,
            "root/alt",
            UiAccessibilityContract::default(),
            "alt_text = 'Alt text'",
        ),
        (
            7,
            "root/tooltip",
            UiAccessibilityContract::default(),
            "tooltip = 'Tooltip text'",
        ),
    ] {
        surface
            .tree
            .insert_child(
                id(1),
                UiTreeNode::new(id(node_id), UiNodePath::new(path))
                    .with_frame(UiFrame::new(4.0, node_id as f32 * 18.0, 80.0, 16.0))
                    .with_template_metadata(UiTemplateNodeMetadata {
                        component: "Button".to_string(),
                        attributes: toml::from_str(attributes).unwrap(),
                        a11y,
                        ..UiTemplateNodeMetadata::default()
                    }),
            )
            .unwrap();
    }
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Explicit")
    );
    assert_eq!(
        snapshot.node(id(4)).unwrap().name.as_deref(),
        Some("Label source")
    );
    assert_eq!(
        snapshot.node(id(5)).unwrap().name.as_deref(),
        Some("Own text")
    );
    assert_eq!(
        snapshot.node(id(6)).unwrap().name.as_deref(),
        Some("Alt text")
    );
    assert_eq!(
        snapshot.node(id(7)).unwrap().name.as_deref(),
        Some("Tooltip text")
    );
}

#[test]
fn labelled_by_uses_higher_id_tooltip_target_without_order_dependency() {
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
                        labelled_by: Some("3".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/tooltip-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 16.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    a11y: UiAccessibilityContract {
                        tooltip: Some("Tooltip label".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(2)).unwrap().name.as_deref(),
        Some("Tooltip label")
    );
}

#[test]
fn hidden_label_references_are_retained_without_visible_children() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 16.0))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata("Label", "text = 'Hidden label'")),
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
                        labelled_by: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Hidden label")
    );
    assert!(snapshot.node(id(2)).unwrap().state.hidden);
    assert!(snapshot.node(id(2)).unwrap().children.is_empty());
    assert!(!snapshot.node(id(1)).unwrap().children.contains(&id(2)));
    assert!(snapshot.diagnostics.iter().all(|diagnostic| {
        diagnostic.code != UiAccessibilityDiagnosticCode::MissingBounds
            || diagnostic.node_id != Some(id(2))
    }));
}

#[test]
fn excluded_hidden_relation_owners_do_not_retain_targets() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/unused-label-target")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden-owner"))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(3)).is_none());
    assert!(snapshot.node(id(2)).is_none());
    assert!(snapshot.node(id(1)).unwrap().children.is_empty());
}

#[test]
fn included_descendants_are_promoted_through_excluded_containers() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(id(1), UiTreeNode::new(id(2), UiNodePath::new("root/slot")))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/slot/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Nested'")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert_eq!(snapshot.node(id(1)).unwrap().children, vec![id(3)]);
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Nested")
    );
}

#[test]
fn hidden_excluded_containers_block_descendant_promotion() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-slot"))
                .with_visibility(UiVisibility::Hidden),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden-slot/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Hidden descendant'")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert!(snapshot.node(id(3)).is_none());
    assert!(!snapshot.node(id(1)).unwrap().children.contains(&id(3)));
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

#[test]
fn accessibility_focus_action_changes_runtime_focus() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/focus-button"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Focus me'")),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Focus);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(result.diagnostics.routed);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.focus")
    );
    assert!(has_note(&result, "status=accepted"));
}

#[test]
fn accessibility_stale_target_rejects_with_status_note() {
    let mut surface = root_surface();
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(404), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, None);
    assert!(has_note(&result, "status=stale_target"));
    assert!(has_note(&result, "code=stale_target"));
}

#[test]
fn accessibility_disabled_activation_rejects_even_when_requested() {
    let mut surface = root_surface();
    let mut disabled_button = state(true, true);
    disabled_button.enabled = false;
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/disabled-button"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 24.0))
                .with_state_flags(disabled_button)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    attributes: toml::from_str("text = 'Disabled'").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Button,
                        actions: vec![
                            UiAccessibilityAction::Activate,
                            UiAccessibilityAction::Focus,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert!(result.component_events.is_empty());
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=disabled_action"));
}

#[test]
fn accessibility_activate_emits_default_commit_component_event() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/activate-button"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Activate'")),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert!(has_note(&result, "status=accepted"));
    assert_eq!(
        result.component_events,
        vec![
            zircon_runtime_interface::ui::dispatch::UiComponentEventReport {
                target: id(2),
                event: UiComponentEvent::Commit {
                    property: "activated".to_string(),
                    value: UiValue::Bool(true),
                },
                delivered: true,
            }
        ]
    );
}

#[test]
fn accessibility_hidden_target_action_rejects_without_component_or_property_mutation() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-input"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_visibility(UiVisibility::Hidden)
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: toml::from_str("text = 'Hidden value'").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        actions: vec![
                            UiAccessibilityAction::Focus,
                            UiAccessibilityAction::SetValue,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        Some("Mutated"),
        None,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=hidden_target"));
    assert!(result.component_events.is_empty());
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("Hidden value"));
}

#[test]
fn accessibility_visible_excluded_target_rejects_without_component_or_property_mutation() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/plain-excluded-child")),
        )
        .unwrap();
    surface.rebuild();
    assert!(surface.tree.node(id(2)).is_some());
    assert!(surface.accessibility_snapshot().node(id(2)).is_none());

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        Some("Mutated"),
        None,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=excluded_target"));
    assert!(result.component_events.is_empty());
    let node = surface.tree.node(id(2)).unwrap();
    assert!(node.template_metadata.is_none());
}

#[test]
fn accessibility_increment_and_decrement_step_slider_value() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/slider"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Slider".to_string(),
                    attributes: toml::from_str("value = 0.5").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Slider,
                        actions: vec![
                            UiAccessibilityAction::Increment,
                            UiAccessibilityAction::Decrement,
                            UiAccessibilityAction::Focus,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        value: Some(UiValue::Float(0.5)),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface.clear_dirty_flags();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Increment);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.adjust_value")
    );
    assert!(has_note(&result, "status=accepted"));
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(
                &event.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value"
                        && matches!(value, UiValue::Float(value) if (*value - 0.51).abs() < f64::EPSILON)
            )
    }));
    let value = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("value")
        .and_then(toml::Value::as_float)
        .unwrap();
    assert!((value - 0.51).abs() < f64::EPSILON);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);

    surface.clear_dirty_flags();
    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Decrement);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.adjust_value")
    );
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(
                &event.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value"
                        && matches!(value, UiValue::Float(value) if (*value - 0.5).abs() < f64::EPSILON)
            )
    }));
    let value = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("value")
        .and_then(toml::Value::as_float)
        .unwrap();
    assert!((value - 0.5).abs() < f64::EPSILON);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
}

#[test]
fn accessibility_dismiss_requires_popup_id() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/dialog"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 80.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dialog".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Dialog,
                        name: Some("Dialog".to_string()),
                        actions: vec![UiAccessibilityAction::Dismiss],
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Dismiss);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(has_note(&result, "status=unsupported"));
    assert!(result
        .diagnostics
        .notes
        .contains(&"accessibility dismiss requires popup id".to_string()));
}

#[test]
fn accessibility_set_value_updates_editable_text_property() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/editable-text"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: toml::from_str("text = 'Old value'").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        actions: vec![
                            UiAccessibilityAction::Focus,
                            UiAccessibilityAction::SetValue,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        value: Some(UiValue::String("Old value".to_string())),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        Some("New value"),
        None,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.set_value")
    );
    assert!(has_note(&result, "status=accepted"));
    assert_eq!(
        result.component_events,
        vec![
            zircon_runtime_interface::ui::dispatch::UiComponentEventReport {
                target: id(2),
                event: UiComponentEvent::ValueChanged {
                    property: "text".to_string(),
                    value: UiValue::String("New value".to_string()),
                },
                delivered: true,
            }
        ]
    );
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("New value"));
}

#[test]
fn accessibility_set_value_without_existing_text_or_value_is_unsupported() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/editable-text-without-value"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: toml::from_str("placeholder = 'Name'").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        actions: vec![
                            UiAccessibilityAction::Focus,
                            UiAccessibilityAction::SetValue,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        value: Some(UiValue::String("".to_string())),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        Some("New value"),
        None,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert!(has_note(&result, "status=unsupported"));
    assert!(has_note(&result, "code=unsupported_role_action"));
    assert!(result.component_events.is_empty());
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert!(!metadata.attributes.contains_key("value"));
    assert!(!metadata.attributes.contains_key("text"));
}
