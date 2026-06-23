use super::*;

#[test]
fn extraction_includes_widget_only_contract_nodes() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/value"))
                .with_frame(UiFrame::new(8.0, 8.0, 96.0, 20.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ValueChip".to_string(),
                    widget: UiWidgetContract {
                        value: Some(UiValue::String("42".to_string())),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("widget-only node included");
    assert_eq!(node.state.value.as_deref(), Some("42"));
}

#[test]
fn extraction_infers_role_and_actions_from_authored_widget_behavior() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/custom-range"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CustomMeter".to_string(),
                    attributes: toml::from_str("value = 0.4").unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Range,
                        value: Some(UiValue::Float(0.4)),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("behavior-authored range node");

    assert_eq!(node.role, UiA11yRole::Slider);
    assert_eq!(node.state.value.as_deref(), Some("0.4"));
    assert!(node.actions.contains(&UiAccessibilityAction::Increment));
    assert!(node.actions.contains(&UiAccessibilityAction::Decrement));
    assert!(node.actions.contains(&UiAccessibilityAction::SetValue));
    assert!(node.actions.contains(&UiAccessibilityAction::Focus));
}

#[test]
fn extraction_reads_value_state_from_runtime_component_state() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/runtime-range"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeRange".to_string(),
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
    surface
        .component_states
        .set_value(id(2), "amount", UiValue::Float(0.75));

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("runtime range value exposed");

    assert_eq!(node.role, UiA11yRole::Slider);
    assert_eq!(node.state.value.as_deref(), Some("0.75"));
    assert!(node.actions.contains(&UiAccessibilityAction::SetValue));
}

#[test]
fn extraction_reads_checked_state_from_runtime_component_state() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/runtime-check"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeToggle".to_string(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface.component_states.set_checked(id(2), true);

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("runtime checked toggle exposed");

    assert_eq!(node.role, UiA11yRole::Checkbox);
    assert_eq!(node.state.checked, Some(UiA11yCheckedState::True));
    assert!(node.actions.contains(&UiAccessibilityAction::Activate));
}

#[test]
fn extraction_reads_checked_state_from_runtime_component_value_alias() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/runtime-check-alias"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeToggle".to_string(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        checked_property: Some("is_on".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
        .component_states
        .set_value(id(2), "is_on", UiValue::Bool(true));

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("runtime checked alias exposed");

    assert_eq!(node.role, UiA11yRole::Checkbox);
    assert_eq!(node.state.checked, Some(UiA11yCheckedState::True));
}

#[test]
fn extraction_reads_selected_state_from_retained_attributes() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/menu-item"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CustomMenuEntry".to_string(),
                    attributes: toml::from_str("selected = true").unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::MenuItem,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("selected menu item exposed");

    assert_eq!(node.role, UiA11yRole::MenuItem);
    assert!(node.state.selected);
    assert!(node.actions.contains(&UiAccessibilityAction::Activate));
}

#[test]
fn extraction_reads_pressed_state_from_retained_active_attribute() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/pressed-button"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(metadata("Button", "text = 'Pressed'\nactive = true")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot.node(id(2)).expect("pressed button exposed");

    assert_eq!(node.role, UiA11yRole::Button);
    assert_eq!(node.state.pressed, Some(true));
    assert!(node.actions.contains(&UiAccessibilityAction::Activate));
}

#[test]
fn extraction_includes_interactive_text_alt_and_explicit_nodes() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Run'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/image"))
                .with_frame(UiFrame::new(10.0, 40.0, 32.0, 32.0))
                .with_template_metadata(metadata("Image", "alt = 'Preview thumbnail'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(4), UiNodePath::new("root/panel"))
                .with_frame(UiFrame::new(100.0, 10.0, 50.0, 50.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Panel,
                        name: Some("Details".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.node(id(2)).unwrap().name.as_deref(), Some("Run"));
    assert_eq!(snapshot.node(id(2)).unwrap().role, UiA11yRole::Button);
    assert!(snapshot
        .node(id(2))
        .unwrap()
        .actions
        .contains(&UiAccessibilityAction::Activate));
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Preview thumbnail")
    );
    assert_eq!(snapshot.node(id(4)).unwrap().role, UiA11yRole::Panel);
    assert_eq!(
        snapshot.node(id(1)).unwrap().children,
        vec![id(2), id(3), id(4)]
    );
}
