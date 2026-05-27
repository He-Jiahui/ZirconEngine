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
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiDispatchEffect,
        UiDispatchHostRequestKind, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiTooltipEffectKind,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
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

fn assert_accessibility_binding_report(
    result: &UiInputDispatchResult,
    expected_applied_count: u64,
) {
    assert_eq!(result.binding_reports.len(), 1);
    let report = &result.binding_reports[0];
    assert_eq!(report.applied_count, expected_applied_count);
    assert_eq!(report.rejected_count, 0);
    assert_eq!(
        report.updates.first().map(|update| update.source.kind),
        Some(UiBindingSourceKind::AccessibilityAction)
    );
}

fn assert_widget_binding_report(result: &UiInputDispatchResult) {
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(
        result
            .binding_reports
            .first()
            .and_then(|report| report.updates.first())
            .map(|update| update.source.kind),
        Some(UiBindingSourceKind::WidgetBehavior)
    );
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

fn insert_runtime_popup_dialog(
    surface: &mut UiSurface,
    open_property: &str,
    actions: Vec<UiAccessibilityAction>,
) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/RuntimePopupDialog"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimePopupDialog".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Dialog,
                        name: Some("RuntimePopupDialog".to_string()),
                        actions,
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some(open_property.to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_popup_menu(surface: &mut UiSurface, open_property: &str) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/RuntimePopupMenu"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimePopupMenu".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Menu,
                        name: Some("RuntimePopupMenu".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some(open_property.to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_tooltip(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/RuntimeTooltip"))
                .with_frame(UiFrame::new(8.0, 8.0, 100.0, 20.0))
                .with_state_flags(state(false, false))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Tooltip".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Tooltip,
                        name: Some("Runtime tooltip".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_menu_item_in_popup_without_item_binding(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/MenuPopup"))
                .with_frame(UiFrame::new(4.0, 4.0, 140.0, 72.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("MenuPopup/ClosePopup", UiEventKind::Click)],
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
            UiTreeNode::new(id(3), UiNodePath::new("root/MenuPopup/Item"))
                .with_frame(UiFrame::new(12.0, 12.0, 100.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeMenuItem".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Runtime menu item".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::MenuItem,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
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
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
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
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
}

#[test]
fn accessibility_expand_sets_runtime_component_disclosure_alias() {
    let mut surface = root_surface();
    insert_runtime_open_widget(
        &mut surface,
        "RuntimeFoldout",
        UiWidgetBehavior::Disclosure,
        "is_open",
    );
    surface.rebuild();

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Expand);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.expand")
    );
    assert_accessibility_binding_report(&result, 2);
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(
                event.event,
                UiComponentEvent::ToggleExpanded { expanded: true }
            )
    }));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["is_open"].as_bool(), Some(true));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("is_open"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("true"));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("expanded disclosure remains exposed")
        .clone();
    assert_eq!(snapshot_node.state.expanded, Some(true));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
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
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
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
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
}

#[test]
fn accessibility_collapse_sets_runtime_component_popup_open_alias() {
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

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Collapse);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.collapse")
    );
    assert_accessibility_binding_report(&result, 2);
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(event.event, UiComponentEvent::ClosePopup)
    }));
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
        .expect("collapsed popup remains exposed")
        .clone();
    assert_eq!(snapshot_node.state.expanded, Some(false));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
}

#[test]
fn accessibility_dismiss_closes_runtime_component_popup_open_alias() {
    let mut surface = root_surface();
    insert_runtime_popup_dialog(
        &mut surface,
        "is_open",
        vec![UiAccessibilityAction::Dismiss],
    );
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Dismiss);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.dismiss")
    );
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_accessibility_binding_report(&result, 2);
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(event.event, UiComponentEvent::ClosePopup)
    }));
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
        .expect("dismissed popup dialog remains exposed")
        .clone();
    assert_eq!(snapshot_node.state.expanded, Some(false));
}

#[test]
fn popup_dialog_default_actions_expose_dismiss_without_expand_collapse() {
    let mut surface = root_surface();
    insert_runtime_popup_dialog(&mut surface, "is_open", Vec::new());
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let snapshot = surface.accessibility_snapshot();
    assert!(
        !snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.node_id == Some(id(2))
                && diagnostic.code == UiAccessibilityDiagnosticCode::UnsupportedRoleAction
        }),
        "dialog popup dismiss should be role/action-compatible: {:?}",
        snapshot.diagnostics
    );
    let snapshot_node = snapshot
        .node(id(2))
        .expect("runtime popup dialog is exposed")
        .clone();
    assert_eq!(snapshot_node.role, UiA11yRole::Dialog);
    assert_eq!(snapshot_node.state.expanded, Some(true));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Dismiss));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Dismiss);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.dismiss")
    );
    assert_accessibility_binding_report(&result, 2);
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(event.event, UiComponentEvent::ClosePopup)
    }));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["is_open"].as_bool(), Some(false));
}

#[test]
fn popup_menu_default_actions_expose_expand_collapse_without_activate() {
    let mut surface = root_surface();
    insert_runtime_popup_menu(&mut surface, "is_open");
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_open", UiValue::Bool(true));

    let snapshot = surface.accessibility_snapshot();
    assert!(
        !snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.node_id == Some(id(2))
                && diagnostic.code == UiAccessibilityDiagnosticCode::UnsupportedRoleAction
        }),
        "menu popup expand/collapse should be role/action-compatible: {:?}",
        snapshot.diagnostics
    );
    let snapshot_node = snapshot
        .node(id(2))
        .expect("runtime popup menu is exposed")
        .clone();
    assert_eq!(snapshot_node.role, UiA11yRole::Menu);
    assert_eq!(snapshot_node.state.expanded, Some(true));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Dismiss));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Collapse);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.collapse")
    );
    assert_accessibility_binding_report(&result, 2);
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(event.event, UiComponentEvent::ClosePopup)
    }));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["is_open"].as_bool(), Some(false));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("collapsed popup menu remains exposed")
        .clone();
    assert_eq!(snapshot_node.state.expanded, Some(false));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Expand));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));
    assert!(!snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Collapse));
}

#[test]
fn accessibility_dismiss_hides_active_runtime_tooltip() {
    let mut surface = root_surface();
    insert_runtime_tooltip(&mut surface);
    surface.rebuild();
    surface.input.show_tooltip("status.hint".to_string());

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("tooltip node is exposed")
        .clone();
    assert_eq!(snapshot_node.role, UiA11yRole::Tooltip);
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Dismiss));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Dismiss);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.dismiss_tooltip")
    );
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(surface.input.tooltip, None);
    assert!(result.applied_effects.iter().any(|applied| matches!(
        applied.effect,
        UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Hide,
            ref tooltip_id,
            ..
        } if tooltip_id == "status.hint"
    )));
    assert!(result.host_requests.iter().any(|request| matches!(
        request.request,
        UiDispatchHostRequestKind::Tooltip {
            kind: UiTooltipEffectKind::Hide,
            ref tooltip_id,
        } if tooltip_id == "status.hint"
    )));
    assert!(result.binding_reports.is_empty());
    assert!(result
        .diagnostics
        .notes
        .contains(&"accessibility_tooltip_hidden:status.hint".to_string()));
}

#[test]
fn accessibility_menu_item_activate_without_item_binding_closes_popup() {
    let mut surface = root_surface();
    insert_runtime_menu_item_in_popup_without_item_binding(&mut surface);
    surface.rebuild();

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(3))
        .expect("menu item node is exposed")
        .clone();
    assert_eq!(snapshot_node.role, UiA11yRole::MenuItem);
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));

    let result = dispatch_accessibility(&mut surface, id(3), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert_widget_binding_report(&result);
    assert!(result.component_events.iter().all(|event| {
        !matches!(
            &event.event,
            UiComponentEvent::Commit { property, .. } if property == "activated"
        )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(event.event, UiComponentEvent::ClosePopup)
    }));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["popup_open"].as_bool(), Some(false));
}
