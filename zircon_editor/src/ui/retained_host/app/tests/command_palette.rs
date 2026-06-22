use super::support::*;

#[test]
fn native_unhandled_ctrl_shift_p_opens_workbench_command_palette() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_native_keymap_palette");

    {
        let host = harness.host.borrow();
        assert!(!workbench_control_bool(
            &host,
            "WorkbenchCommandPalette",
            "popup_open"
        ));
    }

    let dispatch = harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Character("P".into()),
            PhysicalKey::Code(KeyCode::KeyP),
            Some("P"),
            ElementState::Pressed,
        ),
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );

    assert!(
        !dispatch.request_redraw(),
        "keymap dispatch should request redraw through retained host invalidation, not native text damage"
    );

    let mut host = harness.host.borrow_mut();
    host.recompute_if_dirty();
    assert!(workbench_control_bool(
        &host,
        "WorkbenchCommandPalette",
        "popup_open"
    ));
    assert_eq!(
        host.runtime
            .journal()
            .records()
            .last()
            .expect("keymap dispatch should record the command palette event")
            .event,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)
    );
}

#[test]
fn native_command_palette_enter_commits_focused_workbench_command() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_native_palette_enter");

    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Character("P".into()),
            PhysicalKey::Code(KeyCode::KeyP),
            Some("P"),
            ElementState::Pressed,
        ),
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );
    {
        let mut host = harness.host.borrow_mut();
        host.recompute_if_dirty();
        assert!(workbench_control_bool(
            &host,
            "WorkbenchCommandPalette",
            "popup_open"
        ));
    }
    let baseline = harness.journal_len();
    let dispatch = harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Enter),
            PhysicalKey::Code(KeyCode::Enter),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert!(
        dispatch.request_redraw(),
        "native popup acceptance should request repaint for the committed row frame"
    );
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::OpenProject)]
    );
}
