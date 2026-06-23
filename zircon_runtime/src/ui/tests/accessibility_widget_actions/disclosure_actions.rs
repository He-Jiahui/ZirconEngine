use super::*;

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
