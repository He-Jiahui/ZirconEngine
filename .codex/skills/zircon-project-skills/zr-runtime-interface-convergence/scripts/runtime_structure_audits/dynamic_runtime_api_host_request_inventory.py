from __future__ import annotations


HOST_REQUEST_PAYLOAD_ANCHORS = (
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "pub struct ZrRuntimeHostRequestBatchV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "pub enum ZrRuntimeHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "Ime(ZrRuntimeImeHostRequestV1)",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "GamepadRumble(ZrRuntimeGamepadRumbleRequestV1)",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "Cursor(ZrRuntimeCursorHostRequestV1)",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "Clipboard(ZrRuntimeClipboardHostRequestV1)",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "UiAction(ZrRuntimeUiActionHostRequestV1)",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "pub struct ZrRuntimeImeHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "pub struct ZrRuntimeGamepadRumbleRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/host_requests.rs",
        "pub struct ZrRuntimeCursorHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/clipboard.rs",
        "pub struct ZrRuntimeClipboardHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/ui_action.rs",
        "pub struct ZrRuntimeUiActionHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/tests/contracts.rs",
        "runtime_host_request_batch_serializes_ime_requests",
    ),
    (
        "zircon_runtime_interface/src/tests/contracts.rs",
        "runtime_host_request_batch_serializes_gamepad_rumble_requests",
    ),
    (
        "zircon_runtime_interface/src/tests/contracts.rs",
        "runtime_host_request_batch_serializes_cursor_requests",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/host_requests.rs",
        "pub(in crate::dynamic_api) fn runtime_ime_host_request",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/host_requests.rs",
        "pub(in crate::dynamic_api) fn runtime_gamepad_rumble_request",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/host_requests.rs",
        "pub(in crate::dynamic_api) fn runtime_cursor_host_request",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/host_requests.rs",
        "pub(in crate::dynamic_api) fn runtime_clipboard_host_request",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/runtime_ui/action_requests.rs",
        "pub(super) struct RuntimeUiActionRequestQueue",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".drain_ime_host_requests()",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".drain_gamepad_rumble_requests()",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".drain_cursor_host_requests()",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".drain_clipboard_host_requests_into(&mut clipboard_requests)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".drain_action_host_requests_into(&mut action_requests)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::ime)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::gamepad_rumble_v2)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::cursor)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        "ZrRuntimeHostRequestV1::clipboard(runtime_clipboard_host_request(",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::ui_action)",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs",
        "host_request_batch_encodes_runtime_ime_requests",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs",
        "host_request_batch_encodes_gamepad_rumble_requests",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs",
        "host_request_batch_encodes_cursor_requests",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs",
        "host_request_batch_from_bytes(&output)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "ZrRuntimeHostRequestV1::Ime(request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "ZrRuntimeHostRequestV1::GamepadRumble(request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "ZrRuntimeHostRequestV1::Cursor(request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "ZrRuntimeHostRequestV1::Clipboard(request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "ZrRuntimeHostRequestV1::UiAction(request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "apply_runtime_ime_host_request(window.as_ref(), request, native_candidate)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "apply_runtime_gamepad_rumble_request(app, request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "apply_runtime_cursor_host_request(window.as_ref(), request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "apply_runtime_clipboard_host_request(app, event_loop, request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "report_unhandled_runtime_ui_action(app, request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "ZrRuntimeCursorHostRequestKindV1::SetVisible",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "window.set_cursor_visible(request.value)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "ZrRuntimeCursorHostRequestKindV1::SetGrabMode",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "set_cursor_grab(CursorGrabMode::Confined)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "ZrRuntimeCursorHostRequestKindV1::SetHitTest",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "set_cursor_hittest(request.value)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "ZrRuntimeCursorHostRequestKindV1::SetPosition",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs",
        "set_cursor_position",
    ),
    # Current generation-qualified rumble and accepted native IME publication.
    (
        'zircon_runtime/src/dynamic_api/session/host_requests.rs',
        'request: QueuedGamepadRumbleRequest,',
    ),
    (
        'zircon_runtime/src/dynamic_api/session/host_requests.rs',
        'let generation = request.generation();',
    ),
    (
        'zircon_runtime/src/dynamic_api/session/host_requests.rs',
        'ZrRuntimeGamepadRumbleRequestV2::stop(gamepad.0, generation)',
    ),
    (
        'zircon_runtime/src/dynamic_api/session/host_requests.rs',
        'ZrRuntimeGamepadRumbleRequestV2::add(\n                gamepad.0,\n                generation,',
    ),
    (
        'zircon_runtime/src/dynamic_api/session/tests/host_requests.rs',
        'fn encoded_rumble_page_keeps_enqueue_generation_after_reconnect()',
    ),
    (
        'zircon_runtime/src/dynamic_api/session/tests/host_requests.rs',
        'assert_eq!(request.generation, first);',
    ),
    (
        'zircon_runtime/src/dynamic_api/session/tests/host_requests.rs',
        'super::runtime_gamepad_rumble_request(next[0]).generation,\n        second',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs',
        'match app.validate_native_ime_candidate(area) {\n                    Ok(candidate) => Some(candidate),\n                    Err(error) => {\n                        app.reject_native_ime_host_publication();',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs',
        'match apply_runtime_ime_host_request(window.as_ref(), request, native_candidate) {\n                Err(error) => {\n                    if native_candidate.is_some() {\n                        app.reject_native_ime_host_publication();',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs',
        'Ok(()) => {\n                    if let Some(candidate) = native_candidate {\n                        app.record_native_ime_host_publication(candidate);\n                        match app.prepare_native_ime_candidate(',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/mod.rs',
        'if self.native_ime_host_publication != Some(receipt) {',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/mod.rs',
        'self.clear_native_ime_candidate();\n        self.advance_native_ime_generation();',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs',
        '.map(ime_candidate_rect_to_winit)',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs',
        'window.request_ime_update(ImeRequest::Update(',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs',
        'ZrRuntimeHostRequestV1::GamepadRumbleV2(request)',
    ),
    (
        'zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs',
        'runtime_gamepad_rumble_generation_required',
    ),
)
