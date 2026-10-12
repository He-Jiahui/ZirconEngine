pub(in super::super) const EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS: &[(&str, &str)] = &[
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
        "UiHost(ZrRuntimeUiHostRequestV1)",
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
        "zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs",
        "pub struct ZrRuntimeUiHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs",
        "pub enum ZrRuntimeUiHostRequestKindV1",
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
        "zircon_runtime/src/dynamic_api/session/runtime_ui/host_requests.rs",
        "pub(super) struct RuntimeUiHostRequestQueue",
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
        ".drain_ui_host_requests_into(&mut ui_host_requests)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::ime)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::gamepad_rumble)",
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
        "zircon_runtime/src/dynamic_api/session/state.rs",
        ".map(ZrRuntimeHostRequestV1::ui_host)",
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
        "ZrRuntimeHostRequestV1::UiHost(request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "apply_runtime_ime_host_request(window.as_ref(), request)",
    ),
    (
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "apply_runtime_gamepad_rumble_request(request)",
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
        "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs",
        "report_unhandled_runtime_ui_host_request(app, request)",
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
];
