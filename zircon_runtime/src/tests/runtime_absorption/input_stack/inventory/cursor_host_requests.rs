#[test]
fn runtime_12_input_stack_cursor_host_request_anchors_remain_visible() {
    let cursor_host_request_sources = [
        include_str!("../../../../core/framework/input/mod.rs"),
        include_str!("../../../../core/framework/input/input_event.rs"),
        include_str!("../../../../input/runtime/default_input_manager.rs"),
        include_str!("../../../../dynamic_api/session.rs"),
        include_str!("../../../../dynamic_api/session/host_requests.rs"),
        include_str!("../../../../../../zircon_runtime_interface/src/runtime_api/host_requests.rs"),
        include_str!(
            "../../../../../../zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs"
        ),
        include_str!(
            "../../../../../../zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs"
        ),
        include_str!("../../../../platform/tests/backend_tokens.rs"),
        include_str!("../../../../platform/tests/diagnostics.rs"),
    ];
    for required_cursor_anchor in [
        "CursorHostRequest",
        "InputEvent::CursorHostRequest",
        "drain_cursor_host_requests",
        "runtime_cursor_host_request",
        "ZrRuntimeCursorHostRequestV1",
        "ZrRuntimeHostRequestV1::Cursor",
        "apply_runtime_cursor_host_request",
        "set_cursor_visible",
        "set_cursor_grab",
        "set_cursor_hittest",
        "set_cursor_position",
        "platform.cursor_options=supported:winit_window_options",
    ] {
        assert!(
            cursor_host_request_sources
                .iter()
                .any(|source| source.contains(required_cursor_anchor)),
            "Runtime 12 cursor host-request source path should retain `{required_cursor_anchor}`"
        );
    }
}
