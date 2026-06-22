use super::support::*;

#[test]
fn root_menu_pointer_click_dispatches_shared_menu_action_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_menu_dispatch");
    let baseline = harness.journal_len();

    host_context(&harness.root_ui).invoke_menu_pointer_clicked(20.0, 12.0);
    host_context(&harness.root_ui).invoke_menu_pointer_clicked(60.0, 126.0);

    let host = harness.host.borrow();
    assert_eq!(host.menu_pointer_state.open_menu_index, None);
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)]
    );
}

#[test]
fn native_root_menu_pointer_click_dispatches_shared_menu_action_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_native_root_menu_dispatch");
    let baseline = harness.journal_len();

    let open_dispatch = harness
        .root_ui
        .dispatch_native_primary_press_for_test(20.0, 12.0);
    let item_dispatch = harness
        .root_ui
        .dispatch_native_primary_press_for_test(60.0, 126.0);

    assert!(open_dispatch.request_redraw());
    assert!(item_dispatch.request_redraw());
    let host = harness.host.borrow();
    assert_eq!(host.menu_pointer_state.open_menu_index, None);
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)]
    );
}

#[test]
fn root_menu_popup_scroll_and_dismiss_flow_through_shared_pointer_bridge_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_menu_popup_scroll");
    harness
        .root_ui
        .window()
        .set_size(PhysicalSize::new(1280, 220));
    {
        let mut host = harness.host.borrow_mut();
        host.sync_shell_size();
        host.refresh_ui();
    }
    for index in 0..10 {
        harness.dispatch_menu_action(&format!("workbench.layout.preset.save.alpha-{index:02}"));
    }

    let (click_x, click_y) = {
        let host = harness.host.borrow();
        let window_button = host.menu_pointer_layout.button_frames[5];
        (
            window_button.x + window_button.width * 0.5,
            window_button.y + window_button.height * 0.5,
        )
    };

    let baseline = harness.journal_len();
    host_context(&harness.root_ui).invoke_menu_pointer_clicked(click_x, click_y);

    let (popup_scroll_x, popup_scroll_y, dismiss_x, dismiss_y) = {
        let host = harness.host.borrow();
        assert_eq!(host.menu_pointer_state.open_menu_index, Some(5));
        assert!(
            host.menu_pointer_layout.preset_names.len() >= 10,
            "window menu should include saved presets before scroll"
        );
        assert!(
            host.menu_pointer_layout.window_popup_height
                < 72.0 + host.menu_pointer_layout.preset_names.len() as f32 * 30.0,
            "window popup should overflow before scroll"
        );
        let button = host.menu_pointer_layout.button_frames[5];
        let popup_y = button.y + button.height + 3.0;
        (
            button.x + 18.0,
            popup_y + 18.0,
            host.menu_pointer_layout.shell_frame.width - 24.0,
            host.menu_pointer_layout.shell_frame.height - 24.0,
        )
    };

    host_context(&harness.root_ui).invoke_menu_pointer_scrolled(
        popup_scroll_x,
        popup_scroll_y,
        96.0,
    );

    {
        let host = harness.host.borrow();
        assert_eq!(host.menu_pointer_state.open_menu_index, Some(5));
        assert!(host.menu_pointer_state.popup_scroll_offset > 0.0);
    }

    host_context(&harness.root_ui).invoke_menu_pointer_clicked(dismiss_x, dismiss_y);

    let host = harness.host.borrow();
    assert_eq!(host.menu_pointer_state.open_menu_index, None);
    assert!(host.menu_pointer_state.popup_scroll_offset > 0.0);
    assert!(harness.delta_events_since(baseline).is_empty());
}
