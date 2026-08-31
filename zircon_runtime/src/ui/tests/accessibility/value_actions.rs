use super::*;
use zircon_runtime_interface::ui::dispatch::{
    UiNumberInputCommitMethod, UiNumberInputCommitStatus, UiNumberInputParseStatus,
};

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
    assert!(has_note(
        &result,
        "accessibility_binding_source:AccessibilityAction"
    ));
    assert_accessibility_binding_report(&result, 2);
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
    assert!(has_note(
        &result,
        "accessibility_binding_source:AccessibilityAction"
    ));
    assert_accessibility_binding_report(&result, 2);
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
fn accessibility_increment_uses_runtime_component_state_range_contract() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/runtime-range"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeRange".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Slider,
                        actions: vec![
                            UiAccessibilityAction::Increment,
                            UiAccessibilityAction::Decrement,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Range,
                        value_property: Some("amount".to_string()),
                        min_property: Some("low".to_string()),
                        max_property: Some("high".to_string()),
                        step_property: Some("quantum".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "amount", UiValue::Float(0.25));
    surface
        .component_states
        .set_value(id(2), "low", UiValue::Float(0.0));
    surface
        .component_states
        .set_value(id(2), "high", UiValue::Float(1.0));
    surface
        .component_states
        .set_value(id(2), "quantum", UiValue::Float(0.25));

    let result = dispatch_accessibility(&mut surface, id(2), UiAccessibilityAction::Increment);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.adjust_value")
    );
    assert!(has_note(
        &result,
        "accessibility_binding_source:AccessibilityAction"
    ));
    assert_accessibility_binding_report(&result, 2);
    assert!(result.component_events.iter().any(|event| {
        event.target == id(2)
            && event.delivered
            && matches!(
                &event.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "amount"
                        && matches!(value, UiValue::Float(value) if (*value - 0.5).abs() < f64::EPSILON)
            )
    }));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("amount"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("0.5"));
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("adjusted runtime range value remains exposed");
    assert_eq!(node.state.value.as_deref(), Some("0.5"));
}

#[test]
fn accessibility_set_value_uses_widget_value_property_alias() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/custom-meter"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CustomMeter".to_string(),
                    attributes: toml::from_str("amount = 0.25").unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Slider,
                        actions: vec![
                            UiAccessibilityAction::SetValue,
                            UiAccessibilityAction::Focus,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Range,
                        value: Some(UiValue::Float(0.25)),
                        value_property: Some("amount".to_string()),
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
        None,
        Some(0.75),
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(id(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.set_value")
    );
    assert!(has_note(&result, "status=accepted"));
    assert!(has_note(
        &result,
        "accessibility_binding_source:AccessibilityAction"
    ));
    assert!(has_note(&result, "accessibility_binding_updates:applied=2"));
    assert_accessibility_binding_report(&result, 2);
    assert_eq!(
        result.component_events,
        vec![
            zircon_runtime_interface::ui::dispatch::UiComponentEventReport {
                target: id(2),
                event: UiComponentEvent::ValueChanged {
                    property: "amount".to_string(),
                    value: UiValue::Float(0.75),
                },
                delivered: true,
                drag: None,
                template_action: None,
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
    assert_eq!(metadata.attributes["amount"].as_float(), Some(0.75));
    assert!(!metadata.attributes.contains_key("value"));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("amount"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("0.75"));
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("updated range remains exposed");
    assert_eq!(node.state.value.as_deref(), Some("0.75"));
}

#[test]
fn accessibility_set_value_uses_runtime_component_state_value_alias() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/runtime-meter"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeMeter".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Slider,
                        actions: vec![
                            UiAccessibilityAction::SetValue,
                            UiAccessibilityAction::Focus,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Range,
                        value_property: Some("amount".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "amount", UiValue::Float(0.25));

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        None,
        Some(0.75),
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.set_value")
    );
    assert!(has_note(
        &result,
        "accessibility_binding_source:AccessibilityAction"
    ));
    assert!(has_note(&result, "accessibility_binding_updates:applied=2"));
    assert_accessibility_binding_report(&result, 2);
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["amount"].as_float(), Some(0.75));
    let runtime_value = surface
        .component_state(id(2))
        .and_then(|state| state.value("amount"))
        .map(|value| value.display_text());
    assert_eq!(runtime_value.as_deref(), Some("0.75"));
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("updated runtime range");
    assert_eq!(node.state.value.as_deref(), Some("0.75"));
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
    assert!(
        result
            .diagnostics
            .notes
            .contains(&"accessibility dismiss requires popup id".to_string())
    );
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
                    attributes: toml::from_str(
                        "text = 'Old value'\ncaret_offset = 3\nselection_anchor = 1\nselection_focus = 3\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'ld'\ncomposition_restore_text = 'ld'",
                    )
                    .unwrap(),
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
    assert_accessibility_binding_report(&result, 20);
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:caret_offset"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:selection_anchor"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:selection_focus"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_start"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_end"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_text"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_restore_text"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_clauses"
    ));
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
                drag: None,
                template_action: None,
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
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(9));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(9)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(9));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(9)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(9));
    assert_eq!(metadata.attributes["composition_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["composition_restore_text"].as_str(),
        Some("")
    );
    assert_eq!(
        metadata.attributes["composition_clauses"].as_array(),
        Some([].as_slice())
    );
    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("updated text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection::collapsed(9))
    );
}

#[test]
fn accessibility_number_field_set_value_clamps_and_commits_float_atomically() {
    let mut surface = number_field_surface();

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        None,
        Some(120.0),
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.number_input.map(|receipt| (
            receipt.parse_status,
            receipt.commit_method,
            receipt.commit_status,
        )),
        Some((
            UiNumberInputParseStatus::OutOfRange,
            UiNumberInputCommitMethod::Accessibility,
            UiNumberInputCommitStatus::Clamped,
        ))
    );
    let metadata = surface
        .tree
        .node(id(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("number field metadata");
    assert_eq!(metadata.attributes["value"].as_float(), Some(100.0));
    assert_eq!(metadata.attributes["value_text"].as_str(), Some("100"));
    assert_eq!(
        metadata.attributes["number_edit_active"].as_bool(),
        Some(false)
    );
    assert_eq!(
        metadata.attributes["number_value_revision"].as_integer(),
        Some(1)
    );
    assert_eq!(
        metadata.attributes["number_edit_base_revision"].as_integer(),
        Some(1)
    );
    assert!(result.component_events.iter().any(|report| {
        matches!(
            &report.event,
            UiComponentEvent::ValueChanged { property, value }
                if property == "value" && value == &UiValue::Float(100.0)
        )
    }));
}

#[test]
fn accessibility_number_field_rejects_invalid_text_without_partial_state() {
    let mut surface = number_field_surface();

    let result = dispatch_accessibility_with_value(
        &mut surface,
        id(2),
        UiAccessibilityAction::SetValue,
        Some("not-a-number"),
        None,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(
        result
            .diagnostics
            .number_input
            .map(|receipt| receipt.commit_status),
        Some(UiNumberInputCommitStatus::Rejected)
    );
    let metadata = surface
        .tree
        .node(id(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("number field metadata");
    assert_eq!(metadata.attributes["value"].as_float(), Some(42.0));
    assert_eq!(metadata.attributes["value_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["number_edit_active"].as_bool(),
        Some(false)
    );
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}

fn number_field_surface() -> UiSurface {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/number-field"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "NumberField".to_string(),
                    attributes: toml::from_str(
                        "value = 42.0\nvalue_text = ''\nnumber_edit_active = false\nmin = 0.0\nmax = 100.0\nstep = 1.0",
                    )
                    .unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        actions: vec![
                            UiAccessibilityAction::Focus,
                            UiAccessibilityAction::SetValue,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value: Some(UiValue::Float(42.0)),
                        value_property: Some("value".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
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
    assert!(result.binding_reports.is_empty());
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
