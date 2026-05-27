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
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
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
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility.disabled_gate"));
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
    dispatch_accessibility_request(
        surface,
        UiAccessibilityActionRequest {
            target,
            action,
            ..UiAccessibilityActionRequest::default()
        },
    )
}

fn dispatch_accessibility_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    action: UiAccessibilityAction,
    value: &str,
) -> UiInputDispatchResult {
    dispatch_accessibility_request(
        surface,
        UiAccessibilityActionRequest {
            target,
            action,
            value: Some(value.to_string()),
            ..UiAccessibilityActionRequest::default()
        },
    )
}

fn dispatch_accessibility_request(
    surface: &mut UiSurface,
    request: UiAccessibilityActionRequest,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request,
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

fn insert_disabled_popup_with_dismissible_child(surface: &mut UiSurface) {
    let mut disabled_popup_state = state(false, true);
    disabled_popup_state.enabled = false;
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/DisabledPopup"))
                .with_frame(UiFrame::new(4.0, 4.0, 140.0, 72.0))
                .with_state_flags(disabled_popup_state)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "DisabledPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/DisabledPopup/DialogBody"))
                .with_frame(UiFrame::new(12.0, 12.0, 100.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "DialogBody".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Dialog,
                        name: Some("Nested dialog body".to_string()),
                        actions: vec![UiAccessibilityAction::Dismiss],
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_disabled_parent_with_focusable_child(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/DisabledParent"))
                .with_frame(UiFrame::new(4.0, 4.0, 140.0, 72.0))
                .with_state_flags(state(false, false))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Panel".to_string(),
                    attributes: [("disabled".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/DisabledParent/Button"))
                .with_frame(UiFrame::new(12.0, 12.0, 100.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
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
}

fn insert_disabled_parent_with_toggle_child(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/DisabledParent"))
                .with_frame(UiFrame::new(4.0, 4.0, 140.0, 72.0))
                .with_state_flags(state(false, false))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Panel".to_string(),
                    attributes: [("disabled".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/DisabledParent/Toggle"))
                .with_frame(UiFrame::new(12.0, 12.0, 100.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "NestedToggle".to_string(),
                    attributes: [("checked".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    a11y: UiAccessibilityContract {
                        name: Some("Nested toggle".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        checked_property: Some("checked".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_disabled_parent_with_text_input_child(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/DisabledParent"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 72.0))
                .with_state_flags(state(false, false))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Panel".to_string(),
                    attributes: [("disabled".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/DisabledParent/TextInput"))
                .with_frame(UiFrame::new(12.0, 12.0, 120.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "NestedTextInput".to_string(),
                    attributes: [(
                        "text".to_string(),
                        toml::Value::String("Old value".to_string()),
                    )]
                    .into_iter()
                    .collect(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        name: Some("Nested text input".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("text".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

#[test]
fn accessibility_dismiss_from_child_does_not_close_disabled_popup_owner() {
    let mut surface = root_surface();
    insert_disabled_popup_with_dismissible_child(&mut surface);
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "popup_open", UiValue::Bool(true));

    let snapshot = surface.accessibility_snapshot();
    let owner = snapshot.node(id(2)).expect("disabled popup is exposed");
    assert!(owner.state.disabled);
    assert_eq!(owner.actions, vec![UiAccessibilityAction::Focus]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.node_id == Some(id(2))
            && diagnostic.code == UiAccessibilityDiagnosticCode::DisabledAction
    }));
    let child = snapshot
        .node(id(3))
        .expect("dismissible child dialog is exposed");
    assert_eq!(child.role, UiA11yRole::Dialog);
    assert!(child.state.disabled);
    assert_eq!(child.actions, vec![UiAccessibilityAction::Focus]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.node_id == Some(id(3))
            && diagnostic.code == UiAccessibilityDiagnosticCode::DisabledAction
    }));

    let result = dispatch_accessibility(&mut surface, id(3), UiAccessibilityAction::Dismiss);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(3)));
    assert!(result.binding_reports.is_empty());
    assert!(result
        .component_events
        .iter()
        .all(|event| { !matches!(event.event, UiComponentEvent::ClosePopup) }));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.contains("code=disabled_action")));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["popup_open"].as_bool(), Some(true));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("popup_open"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("true"));
}

#[test]
fn accessibility_focus_rejects_disabled_popup_owner() {
    let mut surface = root_surface();
    insert_disabled_popup_with_dismissible_child(&mut surface);
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Focus);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(surface.focus.focused, None);
    assert!(result.binding_reports.is_empty());
    assert!(result.component_events.is_empty());
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.contains("code=focus_rejected")));
}

#[test]
fn accessibility_focus_rejects_child_in_disabled_ancestor() {
    let mut surface = root_surface();
    insert_disabled_parent_with_focusable_child(&mut surface);
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let child = snapshot
        .node(id(3))
        .expect("disabled descendant remains exposed");
    assert!(child.state.disabled);
    assert_eq!(child.actions, vec![UiAccessibilityAction::Focus]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.node_id == Some(id(3))
            && diagnostic.code == UiAccessibilityDiagnosticCode::DisabledAction
    }));

    let result = dispatch_accessibility(&mut surface, id(3), UiAccessibilityAction::Focus);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(3)));
    assert_eq!(surface.focus.focused, None);
    assert!(result.binding_reports.is_empty());
    assert!(result.component_events.is_empty());
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.contains("code=focus_rejected")));
}

#[test]
fn accessibility_activate_rejects_toggle_in_disabled_ancestor_before_mutation() {
    let mut surface = root_surface();
    insert_disabled_parent_with_toggle_child(&mut surface);
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let child = snapshot
        .node(id(3))
        .expect("disabled toggle descendant remains exposed");
    assert!(child.state.disabled);
    assert_eq!(child.actions, vec![UiAccessibilityAction::Focus]);

    let result = dispatch_accessibility(&mut surface, id(3), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(3)));
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=disabled_action"));
    assert!(result.binding_reports.is_empty());
    assert!(result.component_events.is_empty());
    let metadata = surface
        .tree
        .node(id(3))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["checked"].as_bool(), Some(false));
    assert!(surface
        .component_state(id(3))
        .and_then(|state| state.value("checked"))
        .is_none());
}

#[test]
fn accessibility_set_value_rejects_text_input_in_disabled_ancestor_before_mutation() {
    let mut surface = root_surface();
    insert_disabled_parent_with_text_input_child(&mut surface);
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let child = snapshot
        .node(id(3))
        .expect("disabled text input descendant remains exposed");
    assert!(child.state.disabled);
    assert_eq!(child.actions, vec![UiAccessibilityAction::Focus]);

    let result = dispatch_accessibility_value(
        &mut surface,
        id(3),
        UiAccessibilityAction::SetValue,
        "New value",
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.route_target, Some(id(3)));
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=disabled_action"));
    assert!(result.binding_reports.is_empty());
    assert!(result.component_events.is_empty());
    let metadata = surface
        .tree
        .node(id(3))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("Old value"));
    assert!(surface
        .component_state(id(3))
        .and_then(|state| state.value("text"))
        .is_none());
}
