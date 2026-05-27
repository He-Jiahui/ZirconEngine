use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiA11yTextSelection, UiAccessibilityAction, UiAccessibilityContract,
        UiAccessibilityDiagnosticCode,
    },
    component::UiValue,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
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
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility.state_values"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0)),
    );
    surface
}

fn insert_widget_node(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    attributes: &str,
    behavior: UiWidgetBehavior,
) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(node_id, UiNodePath::new(format!("root/{component}")))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    a11y: UiAccessibilityContract {
                        name: Some(component.to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

#[test]
fn extraction_reads_disabled_state_from_runtime_component_value() {
    let mut surface = root_surface();
    insert_widget_node(
        &mut surface,
        id(2),
        "RuntimeButton",
        "",
        UiWidgetBehavior::Button,
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "disabled", UiValue::Bool(true));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("disabled runtime button remains discoverable")
        .clone();

    assert!(snapshot_node.state.disabled);
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Focus));
}

#[test]
fn extraction_inherits_disabled_state_from_retained_parent_attribute() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/disabled-parent"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 64.0))
                .with_state_flags(state(false, false))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Container".to_string(),
                    attributes: toml::from_str("disabled = true").unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/disabled-parent/Button"))
                .with_frame(UiFrame::new(12.0, 12.0, 120.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "NestedButton".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Nested button".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Button,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let nested = snapshot
        .node(id(3))
        .expect("nested button remains discoverable");

    assert!(nested.state.disabled);
    assert_eq!(nested.actions, vec![UiAccessibilityAction::Focus]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.node_id == Some(id(3))
            && diagnostic.code == UiAccessibilityDiagnosticCode::DisabledAction
    }));
}

#[test]
fn extraction_reads_selected_state_from_runtime_component_value() {
    let mut surface = root_surface();
    insert_widget_node(
        &mut surface,
        id(2),
        "RuntimeMenuItem",
        "",
        UiWidgetBehavior::MenuItem,
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "selected", UiValue::Bool(true));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("selected runtime menu item is exposed")
        .clone();

    assert!(snapshot_node.state.selected);
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
}

#[test]
fn unrelated_component_state_does_not_mask_retained_selected_attribute() {
    let mut surface = root_surface();
    insert_widget_node(
        &mut surface,
        id(2),
        "SelectedMenuItem",
        "selected = true",
        UiWidgetBehavior::MenuItem,
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "unrelated", UiValue::Bool(false));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("retained selected menu item is exposed")
        .clone();

    assert!(snapshot_node.state.selected);
}

#[test]
fn extraction_reads_pressed_state_from_runtime_component_value() {
    let mut surface = root_surface();
    insert_widget_node(
        &mut surface,
        id(2),
        "RuntimePressedButton",
        "",
        UiWidgetBehavior::Button,
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "pressed", UiValue::Bool(true));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("pressed runtime button is exposed")
        .clone();

    assert_eq!(snapshot_node.state.pressed, Some(true));
}

#[test]
fn extraction_reads_text_input_selection_from_retained_attributes() {
    let mut surface = root_surface();
    insert_widget_node(
        &mut surface,
        id(2),
        "RetainedTextInput",
        "value = 'hello'\ncaret_offset = 4\nselection_anchor = 1\nselection_focus = 4",
        UiWidgetBehavior::TextInput,
    );
    surface.rebuild();

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("retained text input is exposed")
        .clone();

    assert_eq!(snapshot_node.role, UiA11yRole::TextInput);
    assert_eq!(snapshot_node.state.value.as_deref(), Some("hello"));
    assert_eq!(
        snapshot_node.state.text_selection,
        Some(UiA11yTextSelection {
            caret: 4,
            anchor: 1,
            focus: 4,
        })
    );
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::SetValue));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::ReplaceSelectedText));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::SetTextSelection));
}

#[test]
fn extraction_clamps_text_input_selection_offsets_from_component_state() {
    let mut surface = root_surface();
    insert_widget_node(
        &mut surface,
        id(2),
        "RuntimeTextInput",
        "value = \"a\\u0301b\"",
        UiWidgetBehavior::TextInput,
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "caret_offset", UiValue::Int(2));
    surface
        .component_states
        .set_value(id(2), "selection_anchor", UiValue::Int(3));
    surface
        .component_states
        .set_value(id(2), "selection_focus", UiValue::Int(99));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("runtime text input is exposed")
        .clone();

    let value_len = snapshot_node.state.value.as_deref().unwrap().len();
    assert_eq!(
        snapshot_node.state.text_selection,
        Some(UiA11yTextSelection {
            caret: 1,
            anchor: 3,
            focus: value_len,
        })
    );
}
