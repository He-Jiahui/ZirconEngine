// 核对工作台菜单的键盘导航、首尾跳转、搜索及取消动作。
use super::support::*;

#[test]
fn native_workbench_popup_menu_keyboard_moves_row_hover_and_enter_dispatches_menu_item() {
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
    assert!(menu.popup_open);
    assert!(structured_menu_item(&menu, 3).hovered);

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
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
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let menu = workbench_node(&after, "WorkbenchPopupMenu");
    assert!(structured_menu_item(&menu, 4).hovered);
    assert!(!structured_menu_item(&menu, 3).hovered);

    let enter_result = ui.dispatch_native_popup_enter_for_test();

    assert!(enter_result.request_redraw());
    assert!(enter_result.requires_frame_update());
    assert_eq!(
        clicked_items.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_MENU_MORE_TOOLS_ACTION_ID.to_string()
        )]
    );
}

#[test]
fn native_workbench_popup_menu_home_end_jump_to_boundary_rows() {
    let ui = host_with_componentized_workbench_nodes();

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(menu.popup_open);
    assert!(structured_menu_item(&menu, 3).hovered);

    let home_result = ui.dispatch_native_popup_home_for_test();

    assert!(home_result.request_redraw());
    let after_home = ui.get_host_presentation();
    assert_eq!(
        after_home
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_menu_item"
    );
    assert_eq!(
        after_home
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_NEW_ACTION_ID
    );
    let menu = workbench_node(&after_home, "WorkbenchPopupMenu");
    assert!(structured_menu_item(&menu, 0).hovered);
    assert!(!structured_menu_item(&menu, 3).hovered);

    let end_result = ui.dispatch_native_popup_end_for_test();

    assert!(end_result.request_redraw());
    let after_end = ui.get_host_presentation();
    assert_eq!(
        after_end
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let menu = workbench_node(&after_end, "WorkbenchPopupMenu");
    assert!(!structured_menu_item(&menu, 0).hovered);
    assert!(structured_menu_item(&menu, 4).hovered);
}

#[test]
fn native_workbench_popup_menu_text_search_jumps_to_matching_item() {
    let ui = host_with_componentized_workbench_nodes();

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(menu.popup_open);
    assert!(structured_menu_item(&menu, 3).hovered);

    let search_result = ui.dispatch_native_popup_text_for_test("m");

    assert!(search_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let menu = workbench_node(&after, "WorkbenchPopupMenu");
    assert!(!structured_menu_item(&menu, 3).hovered);
    assert!(structured_menu_item(&menu, 4).hovered);
}

#[test]
fn native_workbench_popup_menu_escape_dispatches_popup_cancel() {
    let ui = host_with_componentized_workbench_nodes();
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
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );

    let escape_result = ui.dispatch_native_popup_escape_for_test();

    assert!(escape_result.request_redraw());
    assert!(escape_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}

#[test]
fn native_popup_no_redraw_search_consumes_f_and_preserves_key_release() {
    use crate::ui::retained_host::UiHostContext;
    use winit::event::{ElementState, KeyEvent};
    use winit::keyboard::{Key, KeyCode, KeyLocation, ModifiersState, PhysicalKey};
    use zircon_runtime_interface::ui::dispatch::UiKeyboardInputState;
    let ui = host_with_componentized_workbench_nodes();
    assert!(ui.dispatch_native_popup_text_for_test("m").request_redraw());
    let unhandled = Rc::new(RefCell::new(Vec::new()));
    let observed = unhandled.clone();
    ui.global::<UiHostContext>()
        .on_unhandled_keyboard_input(move |input| observed.borrow_mut().push(input));
    let key_event = |text: &str, code: KeyCode, state: ElementState| {
        let key = Key::Character(text.into());
        KeyEvent {
            physical_key: PhysicalKey::Code(code),
            logical_key: key.clone(),
            text: Some(text.into()),
            location: KeyLocation::Standard,
            state,
            repeat: false,
            text_with_all_modifiers: Some(text.into()),
            key_without_modifiers: key,
        }
    };
    for (text, code) in [("f", KeyCode::KeyF), ("m", KeyCode::KeyM)] {
        let result = ui.dispatch_native_key_for_test(
            key_event(text, code, ElementState::Pressed),
            ModifiersState::empty(),
        );
        assert!(!result.request_redraw());
        assert!(unhandled.borrow().is_empty());
    }
    ui.dispatch_native_key_for_test(
        key_event("f", KeyCode::KeyF, ElementState::Released),
        ModifiersState::empty(),
    );
    assert_eq!(unhandled.borrow().len(), 1);
    assert_eq!(unhandled.borrow()[0].state, UiKeyboardInputState::Released);
    ui.set_host_presentation(HostWindowPresentationData::default());
    ui.dispatch_native_key_for_test(
        key_event("f", KeyCode::KeyF, ElementState::Pressed),
        ModifiersState::empty(),
    );
    let unhandled = unhandled.borrow();
    assert_eq!(unhandled.len(), 2);
    assert_eq!(unhandled[1].state, UiKeyboardInputState::Pressed);
    assert!(unhandled[1].logical_key.eq_ignore_ascii_case("f"));
}

#[test]
fn native_workbench_popup_menu_outside_primary_press_dispatches_popup_cancel() {
    let ui = host_with_componentized_workbench_nodes();
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    let (x, y) = menu_item_row_point(&menu, 0);
    let move_result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_NEW_ACTION_ID
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
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}
