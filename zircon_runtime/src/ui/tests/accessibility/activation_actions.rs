use super::*;

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
                drag: None,
            }
        ]
    );
}

#[test]
fn accessibility_activate_uses_widget_toggle_behavior_alias() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/favorite"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "FavoritePill".to_string(),
                    attributes: toml::from_str("selected = false").unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        checked_property: Some("selected".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("toggle behavior node is exposed")
        .clone();
    assert_eq!(snapshot_node.role, UiA11yRole::Checkbox);
    assert_eq!(snapshot_node.state.checked, Some(UiA11yCheckedState::False));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result);
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["selected"].as_bool(), Some(true));
    assert!(!metadata.attributes.contains_key("activated"));
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("updated toggle remains exposed");
    assert_eq!(node.state.checked, Some(UiA11yCheckedState::True));
}

#[test]
fn accessibility_activate_uses_runtime_component_checked_value_alias() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/runtime-toggle-alias"))
                .with_frame(UiFrame::new(4.0, 4.0, 96.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeToggle".to_string(),
                    a11y: UiAccessibilityContract::default(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        checked_property: Some("selected".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "selected", UiValue::Bool(true));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("runtime toggle alias is exposed")
        .clone();
    assert_eq!(snapshot_node.state.checked, Some(UiA11yCheckedState::True));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert_widget_binding_report(&result);
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["selected"].as_bool(), Some(false));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("selected"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("false"));
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("updated runtime toggle alias remains exposed");
    assert_eq!(node.state.checked, Some(UiA11yCheckedState::False));
}

#[test]
fn accessibility_activate_uses_widget_disclosure_open_alias() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/foldout"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CustomFoldout".to_string(),
                    attributes: toml::from_str("is_open = false").unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Disclosure,
                        open_property: Some("is_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("disclosure behavior node is exposed")
        .clone();
    assert_eq!(snapshot_node.role, UiA11yRole::Button);
    assert_eq!(snapshot_node.state.expanded, Some(false));
    assert!(snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Activate);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert_widget_binding_report(&result);
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["is_open"].as_bool(), Some(true));
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("updated disclosure remains exposed");
    assert_eq!(node.state.expanded, Some(true));
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
