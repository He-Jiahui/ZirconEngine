use super::*;

#[test]
fn accessibility_dismiss_hides_active_runtime_tooltip() {
    let mut surface = root_surface();
    insert_runtime_tooltip(&mut surface);
    surface.rebuild();
    surface.input.show_tooltip("status.hint".to_string(), None);

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
