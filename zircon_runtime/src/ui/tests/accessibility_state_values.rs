use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityContract},
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
