use super::*;

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
