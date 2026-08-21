use super::support::*;

#[test]
fn componentized_workbench_module_dropdown_open_paints_native_preview_pixels() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .expect("material module tab should dispatch")
        .expect("material module tab should expose a preview binding");
    let dropdown_frame = bridge
        .control_frame("WorkbenchMaterialDomainDropdown")
        .expect("material domain dropdown should have a native frame");
    let closed = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );

    bridge
        .dispatch_control_state("WorkbenchMaterialDomainDropdown", UiEventKind::Change)
        .expect("material domain dropdown should dispatch")
        .expect("material domain dropdown should expose a field binding");
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialDomainDropdown")
            .expect("material domain dropdown projection after open")
            .value_text
            .as_deref(),
        Some("Surface")
    );

    let opened = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    maybe_write_workbench_preview_png(&opened);

    assert!(
        changed_pixel_count_in_frame(&closed, &opened, dropdown_frame) > 0,
        "opening the module dropdown should repaint the native dropdown frame"
    );
    assert!(
        first_non_black_pixel_in_frame(&opened, dropdown_frame).is_some(),
        "opened module dropdown should render visible native pixels"
    );
}

#[test]
fn componentized_workbench_module_dropdown_selection_paints_native_preview_pixels() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .expect("material module tab should dispatch")
        .expect("material module tab should expose a preview binding");
    let dropdown_frame = bridge
        .control_frame("WorkbenchMaterialDomainDropdown")
        .expect("material domain dropdown should have a native frame");
    let before = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );

    assert!(bridge
        .select_dropdown_option("WorkbenchMaterialDomainDropdown", "post_process")
        .expect("material domain dropdown option selection should apply"));
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialDomainDropdown")
            .expect("material domain dropdown projection after option selection")
            .value_text
            .as_deref(),
        Some("post_process")
    );

    let after = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    maybe_write_workbench_preview_png(&after);

    assert!(
        changed_pixel_count_in_frame(&before, &after, dropdown_frame) > 0,
        "selecting a module dropdown option should repaint the native dropdown frame"
    );
    assert!(
        first_non_black_pixel_in_frame(&after, dropdown_frame).is_some(),
        "selected module dropdown should render visible native pixels"
    );
}

#[test]
fn native_workbench_dropdown_option_row_hover_updates_structured_row_state() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let (x, y) = dropdown_option_row_point(&dropdown, 0);
    let result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_option"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );
    let dropdown = workbench_node(&after, "WorkbenchInputDropdown");
    let hovered = structured_option(&dropdown, 0);
    let previous = structured_option(&dropdown, 1);

    assert!(hovered.hovered);
    assert!(!previous.hovered);
    assert!(!previous.focused);
}

#[test]
fn native_workbench_popup_menu_row_hover_updates_structured_row_state() {
    let ui = host_with_componentized_workbench_nodes();
    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(structured_menu_item(&menu, 3).hovered);

    let (x, y) = menu_item_row_point(&menu, 0);
    let result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_menu_item"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_NEW_ACTION_ID
    );
    let menu = workbench_node(&after, "WorkbenchPopupMenu");
    let hovered = structured_menu_item(&menu, 0);
    let previous = structured_menu_item(&menu, 3);

    assert!(hovered.hovered);
    assert!(!previous.hovered);
}

#[test]
fn native_workbench_dropdown_option_primary_press_keeps_selection_path() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let selected_options = Rc::new(RefCell::new(Vec::new()));
    let selected_options_for_callback = selected_options.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_component_showcase_option_selected(move |control_id, action_id, option_id| {
            selected_options_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                option_id.to_string(),
            ));
        });
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    let (x, y) = dropdown_option_row_point(&dropdown, 0);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        selected_options.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            "component_lab.input_dropdown.select".to_string(),
            "dropdown".to_string()
        )]
    );
    assert!(cancelled.borrow().is_empty());
}

#[test]
fn native_workbench_popup_menu_item_primary_press_keeps_menu_selection_path() {
    let ui = host_with_componentized_workbench_nodes();
    let clicked_items = Rc::new(RefCell::new(Vec::new()));
    let clicked_items_for_callback = clicked_items.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            clicked_items_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    let (x, y) = menu_item_row_point(&menu, 0);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        clicked_items.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_MENU_NEW_ACTION_ID.to_string()
        )]
    );
}

#[test]
fn native_workbench_secondary_press_requests_scene_context_menu() {
    let ui = host_with_componentized_workbench_nodes();
    let requests = Rc::new(RefCell::new(Vec::<WorkbenchContextMenuRequestData>::new()));
    let requests_for_callback = requests.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_workbench_context_menu_requested(move |request| {
            requests_for_callback.borrow_mut().push(request);
        });

    let before = ui.get_host_presentation();
    let scene_node = workbench_node(&before, "WorkbenchScenePropsItem");
    let (x, y) = node_right_center(&scene_node);
    let result = ui.dispatch_native_secondary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    let requests = requests.borrow();
    let request = requests
        .first()
        .expect("scene row secondary press should request a context menu");
    assert_eq!(
        request.target_control_id.as_str(),
        "WorkbenchScenePropsItem"
    );
    assert_eq!(
        request.target_path.as_str(),
        "workbench://scene/workbenchscenepropsitem"
    );
    assert_eq!(request.popup_anchor_x, x);
    assert_eq!(request.popup_anchor_y, y);
    assert!(request
        .menu_items
        .iter()
        .any(|item| item.as_str() == "Rename|icon=edit"));
    assert!(request
        .menu_items
        .iter()
        .any(|item| item.as_str() == "Delete|danger,icon=trash"));
}

#[test]
fn native_workbench_disabled_dropdown_option_primary_press_is_ignored_without_cancel() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let selected_options = Rc::new(RefCell::new(Vec::new()));
    let selected_options_for_callback = selected_options.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_component_showcase_option_selected(move |control_id, action_id, option_id| {
            selected_options_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                option_id.to_string(),
            ));
        });
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(structured_option(&dropdown, 2).disabled);
    let (x, y) = dropdown_option_row_point(&dropdown, 2);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(!result.requires_frame_update());
    assert!(selected_options.borrow().is_empty());
    assert!(cancelled.borrow().is_empty());
    let after = ui.get_host_presentation();
    assert!(workbench_node(&after, "WorkbenchInputDropdown").popup_open);
}

#[test]
fn native_workbench_popup_menu_submenu_primary_press_keeps_menu_selection_path() {
    let ui = host_with_componentized_workbench_nodes();
    let clicked_items = Rc::new(RefCell::new(Vec::new()));
    let clicked_items_for_callback = clicked_items.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            clicked_items_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert_eq!(
        structured_menu_item(&menu, 4).action_id.as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let (x, y) = menu_item_row_point(&menu, 4);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        clicked_items.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_MENU_MORE_TOOLS_ACTION_ID.to_string()
        )]
    );
}
