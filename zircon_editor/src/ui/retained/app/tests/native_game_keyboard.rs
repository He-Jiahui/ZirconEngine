use super::support::*;
use crate::core::play::{PlayKind, PlayModeKind, PlayStartRequest};

#[test]
fn escape_without_game_focus_stops_play_and_respects_edit_mode_availability() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_non_game_escape_stop");
    harness.activate_workbench_page();
    {
        let mut host = harness.host.borrow_mut();
        host.runtime.shell().lock().state.enter_play_mode().expect("loaded test world should enter editor play mode");
        host.runtime.play_sessions().request_play(PlayStartRequest::immediate(PlayKind::Play, None)).expect("test play backend should start");
        host.runtime.refresh_reflection();
        host.refresh_ui();
        host.recompute_if_dirty();
        assert!(host.runtime.play_preview_input_active());
        assert!(!host.runtime.play_preview_view_focused().unwrap());
    }
    let before_global_stop = harness.journal_len();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(Key::Named(NamedKey::Escape), PhysicalKey::Code(KeyCode::Escape), None, ElementState::Pressed),
        ModifiersState::empty(),
    );
    assert_eq!(harness.delta_events_since(before_global_stop).iter().filter(|event| matches!(event, EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode))).count(), 1);
    assert_eq!(harness.host.borrow().runtime.play_sessions().mode(), PlayModeKind::Edit);
    let before_edit_escape = harness.journal_len();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(Key::Named(NamedKey::Escape), PhysicalKey::Code(KeyCode::Escape), None, ElementState::Pressed),
        ModifiersState::empty(),
    );
    assert!(harness.delta_events_since(before_edit_escape).iter().all(|event| !matches!(event, EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode))));
}

#[test]
fn focused_game_keeps_regular_keys_in_play_but_dispatches_the_registered_stop_shortcut() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_focused_game_stop_shortcut");
    harness.activate_workbench_page();

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("loaded test world should enter editor play mode");
        host.runtime
            .play_sessions()
            .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
            .expect("test play backend should start");
        {
            let manager = host
                .runtime
                .enter_editor_manager()
                .expect("the original EditorManager should admit Game focus");
            host.runtime
                .shell()
                .lock()
                .focus_play_preview_view(&manager)
                .expect("Game view should receive play focus");
        }
        host.runtime.refresh_reflection();
        host.refresh_ui();
        host.recompute_if_dirty();
        assert!(host.runtime.play_preview_input_active());
        assert!(host.runtime.play_preview_view_focused().unwrap());
    }

    let before_game_key = harness.journal_len();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Character("P".into()),
            PhysicalKey::Code(KeyCode::KeyP),
            Some("P"),
            ElementState::Pressed,
        ),
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );
    assert_eq!(harness.journal_len(), before_game_key);
    assert!(!workbench_control_bool(
        &harness.host.borrow(),
        "WorkbenchCommandPalette",
        "popup_open"
    ));

    {
        let mut host = harness.host.borrow_mut();
        host.workbench_window_bridge.toggle_content_drawer().expect("content drawer should open above Game play input");
        host.refresh_ui();
        host.recompute_if_dirty();
        host.sync_content_drawer_native_focus();
        assert!(host.workbench_window_bridge.content_drawer_is_open());
        assert_eq!(harness.root_ui.global::<UiHostContext>().get_text_input_focus().control_id.as_str(), "SearchEdited");
    }
    let before_drawer_cancel = harness.journal_len();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(Key::Named(NamedKey::Escape), PhysicalKey::Code(KeyCode::Escape), None, ElementState::Pressed),
        ModifiersState::empty(),
    );
    assert_eq!(harness.journal_len(), before_drawer_cancel);
    {
        let host = harness.host.borrow();
        assert!(!host.workbench_window_bridge.content_drawer_is_open());
        assert_eq!(host.runtime.play_sessions().mode(), PlayModeKind::Playing);
        assert!(host.runtime.play_preview_view_focused().unwrap());
    }

    let before_stop = harness.journal_len();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Escape),
            PhysicalKey::Code(KeyCode::Escape),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert_eq!(
        harness.delta_events_since(before_stop),
        vec![EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode)]
    );
    let host = harness.host.borrow();
    assert_eq!(host.runtime.play_sessions().mode(), PlayModeKind::Edit);
    assert_eq!(
        host.runtime.editor_snapshot().unwrap().session_mode,
        EditorSessionMode::Project
    );
    assert!(!host.runtime.play_preview_view_focused().unwrap());
}
