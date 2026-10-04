use zircon_runtime_interface::{
    ProfileControlResponse, RuntimeDiagnosticsSnapshot, RuntimeInputDiagnosticsSnapshot,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
};

use super::{
    mvp_input_probe_counts_advanced, mvp_input_probe_enabled_value, mvp_input_probe_events,
    mvp_input_probe_resize_size, mvp_input_probe_response_received,
    mvp_input_probe_viewport_supported, MVP_INPUT_PROBE_W_KEY_CODE,
};

#[test]
fn input_probe_is_explicitly_opt_in() {
    assert!(!mvp_input_probe_enabled_value(None));
    assert!(!mvp_input_probe_enabled_value(Some("0")));
    assert!(mvp_input_probe_enabled_value(Some("1")));
    assert!(mvp_input_probe_enabled_value(Some("true")));
    assert!(mvp_input_probe_enabled_value(Some("YES")));
}

#[test]
fn input_probe_uses_the_runtime_entry_event_dispatch_boundary() {
    let source = include_str!("../mvp_input_probe.rs");
    let entry_dispatch = ["self.", "dispatch_runtime_event(event_loop, event)"].concat();
    let direct_session_dispatch = ["self.session", ".handle_event(event)"].concat();

    assert!(source.contains(&entry_dispatch));
    assert!(!source.contains(&direct_session_dispatch));
}

#[test]
fn input_probe_covers_viewport_resize_and_restores_it_after_input_events() {
    let events = mvp_input_probe_events(
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(1280, 720),
    );

    assert_eq!(events[0].kind, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1);
    assert_eq!(events[0].size, ZrRuntimeViewportSizeV1::new(640, 360));
    assert_eq!(events[1].kind, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1);
    assert_eq!([events[1].x, events[1].y], [320.0, 180.0]);
    assert_eq!(events[2].kind, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1);
    assert_eq!(events[2].state, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1);
    assert_eq!([events[2].x, events[2].y], [320.0, 180.0]);
    assert_eq!(events[3].kind, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1);
    assert_eq!(events[3].state, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1);
    assert_eq!([events[3].x, events[3].y], [320.0, 180.0]);
    assert_eq!(events[4].kind, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1);
    assert_eq!(events[4].button, ZR_RUNTIME_KEY_ACTION_PRESSED_V1);
    assert_eq!(events[4].key_code, MVP_INPUT_PROBE_W_KEY_CODE);
    assert_eq!(events[5].kind, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1);
    assert_eq!(events[5].button, ZR_RUNTIME_KEY_ACTION_RELEASED_V1);
    assert_eq!(events[5].key_code, MVP_INPUT_PROBE_W_KEY_CODE);
    assert_eq!(events[6].kind, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1);
    assert_eq!(events[6].size, ZrRuntimeViewportSizeV1::new(1280, 720));
}

#[test]
fn input_probe_always_uses_a_distinct_positive_intermediate_viewport() {
    for viewport in [
        ZrRuntimeViewportSizeV1::new(1, 1),
        ZrRuntimeViewportSizeV1::new(1, 720),
        ZrRuntimeViewportSizeV1::new(1280, 1),
        ZrRuntimeViewportSizeV1::new(1280, 720),
    ] {
        let intermediate = mvp_input_probe_resize_size(viewport);

        assert_ne!(intermediate, viewport);
        assert!(intermediate.width > 0);
        assert!(intermediate.height > 0);
    }
}

#[test]
fn input_probe_events_keep_pointer_and_buttons_inside_the_resized_viewport() {
    for viewport in [
        ZrRuntimeViewportSizeV1::new(1, 1),
        ZrRuntimeViewportSizeV1::new(1, 720),
        ZrRuntimeViewportSizeV1::new(1280, 1),
        ZrRuntimeViewportSizeV1::new(1280, 720),
    ] {
        let events = mvp_input_probe_events(ZrRuntimeViewportHandle::new(1), viewport);
        let resized = events[0].size;

        for event in [&events[1], &events[2], &events[3]] {
            assert!(event.x >= 0.0 && event.x <= resized.width as f32);
            assert!(event.y >= 0.0 && event.y <= resized.height as f32);
        }
    }
}

#[test]
fn input_probe_rejects_zero_sized_viewports_before_submitting_events() {
    assert!(mvp_input_probe_viewport_supported(ZrRuntimeViewportSizeV1::new(1, 1)).is_ok());
    assert!(mvp_input_probe_viewport_supported(ZrRuntimeViewportSizeV1::new(0, 1)).is_err());
    assert!(mvp_input_probe_viewport_supported(ZrRuntimeViewportSizeV1::new(1, 0)).is_err());
}

#[test]
fn input_probe_requires_the_requested_events_to_advance_each_input_counter() {
    let before = RuntimeInputDiagnosticsSnapshot {
        viewport_resize_count: 7,
        pointer_move_count: 7,
        mouse_button_press_count: 7,
        mouse_button_release_count: 7,
        keyboard_press_count: 7,
        keyboard_release_count: 7,
    };

    let error = mvp_input_probe_counts_advanced(&before, &before)
        .expect_err("pre-existing input activity must not satisfy the MVP probe");

    assert!(error.contains("pointer_move_count before=7 after=7"));
    assert!(error.contains("viewport_resize_count before=7 after=7"));
    assert!(error.contains("mouse_button_press_count before=7 after=7"));
    assert!(error.contains("mouse_button_release_count before=7 after=7"));
    assert!(error.contains("keyboard_press_count before=7 after=7"));
    assert!(error.contains("keyboard_release_count before=7 after=7"));

    let missing_restore = RuntimeInputDiagnosticsSnapshot {
        viewport_resize_count: 8,
        pointer_move_count: 8,
        mouse_button_press_count: 8,
        mouse_button_release_count: 8,
        keyboard_press_count: 8,
        keyboard_release_count: 8,
    };

    let error = mvp_input_probe_counts_advanced(&before, &missing_restore)
        .expect_err("the restore viewport event must advance the resize counter");

    assert!(error.contains("viewport_resize_count before=7 after=8 required_delta=2"));

    let after = RuntimeInputDiagnosticsSnapshot {
        viewport_resize_count: 9,
        pointer_move_count: 8,
        mouse_button_press_count: 8,
        mouse_button_release_count: 8,
        keyboard_press_count: 8,
        keyboard_release_count: 8,
    };
    assert!(mvp_input_probe_counts_advanced(&before, &after).is_ok());
}

#[test]
fn input_probe_rejects_non_ok_runtime_diagnostics_response() {
    let mut response = ProfileControlResponse::error("input manager unavailable");
    response.runtime_diagnostics = Some(RuntimeDiagnosticsSnapshot {
        input: RuntimeInputDiagnosticsSnapshot {
            viewport_resize_count: 1,
            pointer_move_count: 1,
            mouse_button_press_count: 1,
            mouse_button_release_count: 1,
            keyboard_press_count: 1,
            keyboard_release_count: 1,
        },
        ..RuntimeDiagnosticsSnapshot::default()
    });

    assert_eq!(
        mvp_input_probe_response_received(&response).unwrap_err(),
        "runtime input diagnostics request reported status=error message=input manager unavailable"
    );
}
