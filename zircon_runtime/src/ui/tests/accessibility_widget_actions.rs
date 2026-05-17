use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityContract,
    },
    component::UiValue,
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata,
    },
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
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility.widget_actions"));
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
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target,
                    action,
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap()
}

fn insert_runtime_open_widget(
    surface: &mut UiSurface,
    component: &str,
    behavior: UiWidgetBehavior,
    open_property: &str,
) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new(format!("root/{component}")))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some(component.to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior,
                        open_property: Some(open_property.to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

#[test]
fn extraction_reads_expanded_state_from_runtime_component_open_alias() {
    let mut surface = root_surface();
    insert_runtime_open_widget(
        &mut surface,
        "RuntimeFoldout",
        UiWidgetBehavior::Disclosure,
        "is_open",
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("runtime disclosure alias is exposed")
        .clone();

    assert_eq!(snapshot_node.role, UiA11yRole::Button);
    assert_eq!(snapshot_node.state.expanded, Some(true));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
}

#[test]
fn accessibility_activate_uses_runtime_component_open_alias() {
    let mut surface = root_surface();
    insert_runtime_open_widget(
        &mut surface,
        "RuntimeFoldout",
        UiWidgetBehavior::Disclosure,
        "is_open",
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["is_open"].as_bool(), Some(false));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("is_open"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("false"));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("updated runtime disclosure alias remains exposed")
        .clone();
    assert_eq!(snapshot_node.state.expanded, Some(false));
}

#[test]
fn extraction_reads_popup_state_from_runtime_component_open_alias() {
    let mut surface = root_surface();
    insert_runtime_open_widget(
        &mut surface,
        "RuntimePopup",
        UiWidgetBehavior::Popup,
        "is_open",
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("runtime popup alias is exposed")
        .clone();

    assert_eq!(snapshot_node.role, UiA11yRole::Button);
    assert_eq!(snapshot_node.state.expanded, Some(true));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
}

#[test]
fn accessibility_activate_uses_runtime_component_popup_open_alias() {
    let mut surface = root_surface();
    insert_runtime_open_widget(
        &mut surface,
        "RuntimePopup",
        UiWidgetBehavior::Popup,
        "is_open",
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["is_open"].as_bool(), Some(false));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("is_open"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("false"));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("updated runtime popup alias remains exposed")
        .clone();
    assert_eq!(snapshot_node.state.expanded, Some(false));
}
