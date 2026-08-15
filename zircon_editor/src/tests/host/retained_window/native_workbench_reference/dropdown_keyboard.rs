use super::support::*;

#[test]
fn native_workbench_dropdown_keyboard_moves_row_hover_and_enter_dispatches_option() {
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

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
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
    assert!(structured_option(&dropdown, 0).hovered);
    assert!(!structured_option(&dropdown, 1).hovered);

    let enter_result = ui.dispatch_native_popup_enter_for_test();

    assert!(enter_result.request_redraw());
    assert!(enter_result.requires_frame_update());
    assert_eq!(
        selected_options.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            "component_lab.input_dropdown.select".to_string(),
            "dropdown".to_string()
        )]
    );
}

#[test]
fn native_workbench_dropdown_home_jumps_to_first_enabled_option() {
    let ui = host_with_open_workbench_dropdown_nodes();

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let home_result = ui.dispatch_native_popup_home_for_test();

    assert!(home_result.request_redraw());
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
    assert!(structured_option(&dropdown, 0).hovered);
    assert!(!structured_option(&dropdown, 1).hovered);
}

#[test]
fn native_workbench_dropdown_text_search_jumps_to_matching_enabled_option() {
    let ui = host_with_open_workbench_dropdown_nodes();

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let search_result = ui.dispatch_native_popup_text_for_test("d");

    assert!(search_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );
    let dropdown = workbench_node(&after, "WorkbenchInputDropdown");
    assert!(structured_option(&dropdown, 0).hovered);
    assert!(!structured_option(&dropdown, 1).hovered);
}

#[test]
fn native_workbench_dropdown_escape_dispatches_popup_cancel() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );

    let escape_result = ui.dispatch_native_popup_escape_for_test();

    assert!(escape_result.request_redraw());
    assert!(escape_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}

#[test]
fn native_workbench_dropdown_outside_primary_press_dispatches_popup_cancel() {
    let ui = host_with_open_workbench_dropdown_nodes();
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
    let move_result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );

    let outside_result = ui.dispatch_native_primary_press_for_test(
        OUTSIDE_WORKBENCH_POPUP_X,
        OUTSIDE_WORKBENCH_POPUP_Y,
    );

    assert!(outside_result.request_redraw());
    assert!(outside_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}
