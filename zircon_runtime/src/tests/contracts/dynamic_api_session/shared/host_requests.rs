const HOST_API: &str = "zircon_runtime_interface/src/runtime_api/host/host_requests.rs";
const UI_HOST_API: &str = "zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs";
const INTERFACE_TESTS: &str = "zircon_runtime_interface/src/tests/contracts.rs";
const SESSION_REQUESTS: &str = "zircon_runtime/src/dynamic_api/session/host_requests.rs";
const SESSION_STATE: &str = "zircon_runtime/src/dynamic_api/session/state.rs";
const PAYLOAD_TESTS: &str = "zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs";
const APP_ROUTING: &str = "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs";
const CURSOR_REQUEST: &str =
    "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs";

pub(in super::super) const EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS: &[(&str, &str)] = &[
    (HOST_API, "pub struct ZrRuntimeHostRequestBatchV1"),
    (HOST_API, "pub enum ZrRuntimeHostRequestV1"),
    (HOST_API, "Ime(ZrRuntimeImeHostRequestV1)"),
    (HOST_API, "GamepadRumble(ZrRuntimeGamepadRumbleRequestV1)"),
    (HOST_API, "Cursor(ZrRuntimeCursorHostRequestV1)"),
    (HOST_API, "Clipboard(ZrRuntimeClipboardHostRequestV1)"),
    (HOST_API, "UiAction(ZrRuntimeUiActionHostRequestV1)"),
    (HOST_API, "UiHost(ZrRuntimeUiHostRequestV1)"),
    (HOST_API, "pub struct ZrRuntimeImeHostRequestV1"),
    (HOST_API, "pub struct ZrRuntimeGamepadRumbleRequestV1"),
    (HOST_API, "pub struct ZrRuntimeCursorHostRequestV1"),
    (
        "zircon_runtime_interface/src/runtime_api/host/clipboard.rs",
        "pub struct ZrRuntimeClipboardHostRequestV1",
    ),
    (
        "zircon_runtime_interface/src/runtime_api/host/ui_action.rs",
        "pub struct ZrRuntimeUiActionHostRequestV1",
    ),
    (UI_HOST_API, "pub struct ZrRuntimeUiHostRequestV1"),
    (UI_HOST_API, "pub enum ZrRuntimeUiHostRequestKindV1"),
    (
        INTERFACE_TESTS,
        "runtime_host_request_batch_serializes_ime_requests",
    ),
    (
        INTERFACE_TESTS,
        "runtime_host_request_batch_serializes_gamepad_rumble_requests",
    ),
    (
        INTERFACE_TESTS,
        "runtime_host_request_batch_serializes_cursor_requests",
    ),
    (
        SESSION_REQUESTS,
        "pub(in crate::dynamic_api) fn runtime_ime_host_request",
    ),
    (
        SESSION_REQUESTS,
        "pub(in crate::dynamic_api) fn runtime_gamepad_rumble_request",
    ),
    (
        SESSION_REQUESTS,
        "pub(in crate::dynamic_api) fn runtime_cursor_host_request",
    ),
    (
        SESSION_REQUESTS,
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
    (SESSION_STATE, ".drain_ime_host_requests()"),
    (SESSION_STATE, ".drain_gamepad_rumble_requests()"),
    (SESSION_STATE, ".drain_cursor_host_requests()"),
    (
        SESSION_STATE,
        ".drain_clipboard_host_requests_into(&mut clipboard_requests)",
    ),
    (
        SESSION_STATE,
        ".drain_action_host_requests_into(&mut action_requests)",
    ),
    (
        SESSION_STATE,
        ".drain_ui_host_requests_into(&mut ui_host_requests)",
    ),
    (SESSION_STATE, ".map(ZrRuntimeHostRequestV1::ime)"),
    (
        SESSION_STATE,
        ".map(ZrRuntimeHostRequestV1::gamepad_rumble_v2)",
    ),
    (SESSION_STATE, ".map(ZrRuntimeHostRequestV1::cursor)"),
    (
        SESSION_STATE,
        "ZrRuntimeHostRequestV1::clipboard(runtime_clipboard_host_request(",
    ),
    (SESSION_STATE, ".map(ZrRuntimeHostRequestV1::ui_action)"),
    (SESSION_STATE, ".map(ZrRuntimeHostRequestV1::ui_host)"),
    (
        PAYLOAD_TESTS,
        "host_request_batch_encodes_runtime_ime_requests",
    ),
    (
        PAYLOAD_TESTS,
        "host_request_batch_encodes_gamepad_rumble_requests",
    ),
    (PAYLOAD_TESTS, "host_request_batch_encodes_cursor_requests"),
    (PAYLOAD_TESTS, "host_request_batch_from_bytes(&output)"),
    (APP_ROUTING, "ZrRuntimeHostRequestV1::Ime(request)"),
    (
        APP_ROUTING,
        "ZrRuntimeHostRequestV1::GamepadRumble(request)",
    ),
    (APP_ROUTING, "ZrRuntimeHostRequestV1::Cursor(request)"),
    (APP_ROUTING, "ZrRuntimeHostRequestV1::Clipboard(request)"),
    (APP_ROUTING, "ZrRuntimeHostRequestV1::UiAction(request)"),
    (APP_ROUTING, "ZrRuntimeHostRequestV1::UiHost(request)"),
    (
        APP_ROUTING,
        "apply_runtime_ime_host_request(window.as_ref(), request)",
    ),
    (APP_ROUTING, "apply_runtime_gamepad_rumble_request(request)"),
    (
        APP_ROUTING,
        "apply_runtime_cursor_host_request(window.as_ref(), request)",
    ),
    (
        APP_ROUTING,
        "apply_runtime_clipboard_host_request(app, event_loop, request)",
    ),
    (
        APP_ROUTING,
        "report_unhandled_runtime_ui_action(app, request)",
    ),
    (
        APP_ROUTING,
        "report_unhandled_runtime_ui_host_request(app, request)",
    ),
    (
        CURSOR_REQUEST,
        "ZrRuntimeCursorHostRequestKindV1::SetVisible",
    ),
    (CURSOR_REQUEST, "window.set_cursor_visible(request.value)"),
    (
        CURSOR_REQUEST,
        "ZrRuntimeCursorHostRequestKindV1::SetGrabMode",
    ),
    (CURSOR_REQUEST, "set_cursor_grab(CursorGrabMode::Confined)"),
    (
        CURSOR_REQUEST,
        "ZrRuntimeCursorHostRequestKindV1::SetHitTest",
    ),
    (CURSOR_REQUEST, "set_cursor_hittest(request.value)"),
    (
        CURSOR_REQUEST,
        "ZrRuntimeCursorHostRequestKindV1::SetPosition",
    ),
    (CURSOR_REQUEST, "set_cursor_position"),
];
