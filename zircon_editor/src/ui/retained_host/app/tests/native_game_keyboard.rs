use super::support::*;
use crate::core::play::{PlayKind, PlayModeKind, PlayStartRequest};

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
        host.runtime
            .shell()
            .lock()
            .focus_play_preview_view()
            .expect("Game view should receive play focus");
        host.runtime.refresh_reflection();
        host.refresh_ui();
        host.recompute_if_dirty();
        assert!(host.runtime.play_preview_input_active());
        assert!(host.runtime.play_preview_view_focused());
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

    let before_stop = harness.journal_len();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::F5),
            PhysicalKey::Code(KeyCode::F5),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::SHIFT,
    );

    assert_eq!(
        harness.delta_events_since(before_stop),
        vec![EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode)]
    );
    let host = harness.host.borrow();
    assert_eq!(host.runtime.play_sessions().mode(), PlayModeKind::Edit);
    assert_eq!(
        host.runtime.editor_snapshot().session_mode,
        EditorSessionMode::Project
    );
    assert!(!host.runtime.play_preview_view_focused());
}
