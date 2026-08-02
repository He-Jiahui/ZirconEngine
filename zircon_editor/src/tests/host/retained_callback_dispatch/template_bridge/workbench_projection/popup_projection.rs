use super::*;

#[test]
fn componentized_workbench_window_projection_exports_dropdown_and_popup_rows() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));

    let dropdown = template_contract_node(&nodes, "WorkbenchInputDropdown");
    assert_eq!(dropdown.role.as_str(), "Dropdown");
    assert_eq!(dropdown.layout_offset_x, 0.0);
    assert_eq!(dropdown.layout_offset_y, 0.0);
    assert_eq!(dropdown.frame.height, 32.0);
    assert_eq!(
        dropdown.options_text.as_str(),
        "dropdown, option_a, option_b"
    );
    assert_eq!(dropdown.options.row_count(), 3);
    assert_eq!(dropdown.options.row_data(0).as_deref(), Some("dropdown"));
    assert_eq!(dropdown.structured_options.row_count(), 3);

    let selected = template_contract_option(&dropdown.structured_options, 0);
    assert_eq!(selected.id.as_str(), "dropdown");
    assert_eq!(selected.label.as_str(), "dropdown");
    assert!(selected.selected);
    assert!(selected.special);
    assert!(!selected.disabled);

    let hovered = template_contract_option(&dropdown.structured_options, 1);
    assert_eq!(hovered.id.as_str(), "option_a");
    assert!(hovered.focused);
    assert!(hovered.hovered);
    assert!(!hovered.selected);

    let disabled = template_contract_option(&dropdown.structured_options, 2);
    assert_eq!(disabled.id.as_str(), "option_b");
    assert!(disabled.disabled);

    let stepper = template_contract_node(&nodes, "WorkbenchInputStepper");
    assert_eq!(stepper.role.as_str(), "InputField");
    assert_eq!(stepper.layout_offset_x, 0.0);
    assert_eq!(stepper.layout_offset_y, 0.0);
    assert_eq!(stepper.frame.height, 32.0);

    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    assert_eq!(popup_menu.role.as_str(), "Menu");
    assert!(popup_menu.popup_open);
    assert_eq!(popup_menu.frame.width, 145.0);
    assert_eq!(popup_menu.layout_offset_y, -12.0);
    assert_eq!(popup_menu.structured_menu_items.row_count(), 5);

    let delete = template_contract_menu_item(&popup_menu.structured_menu_items, 3);
    assert_eq!(delete.raw.as_str(), "Delete|danger,hovered,icon=trash");
    assert_eq!(delete.label.as_str(), "Delete");
    assert_eq!(delete.action_id.as_str(), "menu.item.delete");
    assert!(delete.hovered);
    assert!(!delete.disabled);

    let more_tools = template_contract_menu_item(&popup_menu.structured_menu_items, 4);
    assert_eq!(more_tools.raw.as_str(), "More Tools|submenu");
    assert_eq!(more_tools.label.as_str(), "More Tools");
    assert_eq!(more_tools.action_id.as_str(), "menu.item.more_tools");
    assert!(!more_tools.hovered);
    assert!(!more_tools.disabled);
}

#[test]
fn componentized_workbench_dropdown_option_selection_updates_value_and_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_dropdown_select");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .unwrap()
        .expect("dropdown should expose an open binding");
    assert!(control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));

    let effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchInputDropdown",
        "option_a",
    )
    .expect("dropdown option selection should dispatch");

    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value").as_deref(),
        Some("option_a")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value_text").as_deref(),
        Some("option_a")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "selected"));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: "component_lab.input_dropdown.select".to_string(),
            pressed: false,
        })
    );
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_group
            .as_deref(),
        Some("ComponentLabPreview")
    );

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let dropdown = template_contract_node(&nodes, "WorkbenchInputDropdown");
    let old_selected = template_contract_option(&dropdown.structured_options, 0);
    assert_eq!(old_selected.id.as_str(), "dropdown");
    assert!(!old_selected.selected);
    assert!(!old_selected.special);

    let option = template_contract_option(&dropdown.structured_options, 1);
    assert_eq!(option.id.as_str(), "option_a");
    assert!(option.selected);
    assert!(option.special);
    assert!(!option.focused);
    assert!(!option.hovered);

    let disabled = template_contract_option(&dropdown.structured_options, 2);
    assert_eq!(disabled.id.as_str(), "option_b");
    assert!(disabled.disabled);

    let no_effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchInputDropdown",
        "option_b",
    )
    .expect("disabled option selection should be swallowed");
    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value").as_deref(),
        Some("option_a")
    );
    assert_eq!(harness.runtime.journal().records().len(), 1);
    assert_eq!(no_effects, UiHostEventEffects::default());
}

#[test]
fn componentized_workbench_popup_cancel_closes_dropdown_without_value_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .unwrap()
        .expect("dropdown should expose an open binding");
    assert!(control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));

    let effects = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchInputDropdown",
        WORKBENCH_POPUP_CANCEL_ACTION_ID,
    )
    .expect("popup cancel action should be routed")
    .expect("popup cancel should close the dropdown");

    assert!(!control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "selected"));
    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value").as_deref(),
        Some("dropdown")
    );
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let dropdown = template_contract_node(&nodes, "WorkbenchInputDropdown");
    let selected = template_contract_option(&dropdown.structured_options, 0);
    assert_eq!(selected.id.as_str(), "dropdown");
    assert!(selected.selected);
    assert!(selected.special);
    assert!(!selected.focused);
    assert!(!selected.hovered);
    assert!(!selected.pressed);
    let next = template_contract_option(&dropdown.structured_options, 1);
    assert!(!next.focused);
    assert!(!next.hovered);
    assert!(!next.pressed);

    let no_effects = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchInputDropdown",
        WORKBENCH_POPUP_CANCEL_ACTION_ID,
    )
    .expect("popup cancel action should still be routed when closed")
    .expect("closed dropdown cancel should be a no-op");
    assert_eq!(no_effects, UiHostEventEffects::default());
}

#[test]
fn componentized_workbench_popup_menu_item_selection_updates_value_and_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_menu_select");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchPopupMenu",
        "menu.item.delete",
    )
    .expect("popup menu item should be handled")
    .expect("popup menu item selection should dispatch");

    assert_eq!(
        control_string(&bridge, "WorkbenchPopupMenu", "value").as_deref(),
        Some("Delete")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchPopupMenu", "value_text").as_deref(),
        Some("Delete")
    );
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "selected"));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(harness.runtime.journal().records().is_empty());

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    let delete = template_contract_menu_item(&popup_menu.structured_menu_items, 3);
    assert_eq!(delete.raw.as_str(), "Delete|danger,icon=trash");
    assert_eq!(delete.action_id.as_str(), "menu.item.delete");
    assert!(!delete.hovered);
    assert!(!delete.pressed);

    assert!(dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchPopupMenu",
        "Missing"
    )
    .is_none());
}

#[test]
fn componentized_workbench_popup_cancel_closes_menu_without_selecting_item() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));

    let effects = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchPopupMenu",
        WORKBENCH_POPUP_CANCEL_ACTION_ID,
    )
    .expect("popup cancel action should be routed")
    .expect("popup cancel should close the menu");

    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "selected"));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    let delete = template_contract_menu_item(&popup_menu.structured_menu_items, 3);
    assert_eq!(delete.raw.as_str(), "Delete|danger,icon=trash");
    assert_eq!(delete.action_id.as_str(), "menu.item.delete");
    assert!(!delete.focused);
    assert!(!delete.hovered);
    assert!(!delete.pressed);

    let no_route = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchPopupMenu",
        "WrongAction",
    );
    assert!(no_route.is_none());
}

#[test]
fn componentized_workbench_pointer_focuses_input_fields_without_authored_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_input_focus");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(!control_component_focused(&bridge, "WorkbenchInputText"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#101417")
    );

    let text_point = control_center(&bridge, "WorkbenchInputText");
    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, text_point)
            .with_button(UiPointerButton::Primary),
    )
    .expect("text input pointer press should request paint-only feedback")
    .unwrap();

    assert!(control_component_focused(&bridge, "WorkbenchInputText"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#151b1f")
    );
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#2a3238")
    );
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let stepper_point = control_center(&bridge, "WorkbenchInputStepper");
    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, stepper_point)
            .with_button(UiPointerButton::Primary),
    )
    .expect("stepper input pointer press should request paint-only feedback")
    .unwrap();

    assert!(!control_component_focused(&bridge, "WorkbenchInputText"));
    assert!(control_component_focused(&bridge, "WorkbenchInputStepper"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#101417")
    );
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputStepper").as_deref(),
        Some("#151b1f")
    );
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchInputStepper").as_deref(),
        Some("#2a3238")
    );
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchCheckboxOn", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRadioOn", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchRadioOff", "checked"));
    assert!(control_bool(&bridge, "WorkbenchToggleOn", "checked"));
    assert!(control_bool(&bridge, "WorkbenchListSelected", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLabsTabOne", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchLabsTabTwo", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableItem", "selected"));
    assert!(control_bool(&bridge, "WorkbenchTableSelected", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableTail", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchLabsTabTwo", UiEventKind::Click)
            .unwrap()
            .expect("labs tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.labs_tab_two.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchLabsTabOne", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLabsTabTwo", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLabsTabTwo", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchLabsTabThree", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchCheckboxOff", UiEventKind::Toggle)
            .unwrap()
            .expect("checkbox should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.checkbox_off.toggle"
    ));
    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchCheckboxOff").as_deref(),
        Some("#173942")
    );

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchRadioOff", UiEventKind::Change)
            .unwrap()
            .expect("radio should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.radio_off.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchRadioOn", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRadioOff", "checked"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchToggleOn", UiEventKind::Toggle)
            .unwrap()
            .expect("switch should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.switch.toggle"
    ));
    assert!(!control_bool(&bridge, "WorkbenchToggleOn", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchToggleOn", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchListItem", UiEventKind::Click)
            .unwrap()
            .expect("list item should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.list_item.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchListItem", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchListSelected", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchListItem").as_deref(),
        Some("#12383d")
    );

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchTableItem", UiEventKind::Click)
            .unwrap()
            .expect("table item should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.table_item.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchTableItem", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableSelected", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableTail", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchTableTail", UiEventKind::Click)
            .unwrap()
            .expect("table tail should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.table_tail.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchTableItem", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableSelected", "selected"));
    assert!(control_bool(&bridge, "WorkbenchTableTail", "selected"));
}

#[test]
fn startup_template_runtime_loads_componentized_workbench_window_bridge_source() {
    let _guard = env_lock().lock().unwrap();

    let runtime = Arc::new(load_startup_builtin_template_runtime().unwrap());
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(
        runtime,
        UiSize::new(1672.0, 941.0),
    )
    .unwrap();

    assert!(bridge
        .host_projection()
        .node_by_control_id(EditorWorkbenchTemplateControlIds::ROOT)
        .is_some());
    assert_eq!(
        bridge.control_frame(EditorWorkbenchTemplateControlIds::STATUS_BAR),
        Some(UiFrame::new(0.0, 895.0, 1672.0, 46.0))
    );
}
