use super::super::source_assertions::assert_source_order;

#[test]
fn runtime_entry_applies_runtime_ime_host_requests_from_session_drain() {
    let frame_loop_source = include_str!("../../runtime_entry_app/frame_loop.rs");
    let drain_source = include_str!("../../runtime_entry_app/host_requests/drain.rs");
    let routing_source = include_str!("../../runtime_entry_app/host_requests/routing.rs");
    let ime_mod_source = include_str!("../../runtime_entry_app/host_requests/ime/mod.rs");
    let ime_request_source = include_str!("../../runtime_entry_app/host_requests/ime/request.rs");
    let ime_geometry_source = include_str!("../../runtime_entry_app/host_requests/ime/geometry.rs");

    assert_source_order(
        frame_loop_source,
        &[
            "self.frame_cadence.take_frame_request(now)",
            "self.session.tick_frame()",
            "self.apply_runtime_host_requests(event_loop)",
            "window.request_redraw();",
        ],
        "runtime frame loop should apply host requests after runtime tick and before redraw",
    );
    assert_source_order(
        drain_source,
        &[
            "self.session.drain_host_requests()",
            "if !requests.is_empty()",
            "clear_finished_rumble_effects(",
            "for request in requests",
            "apply_runtime_host_request(self, event_loop, request);",
        ],
        "runtime entry host-request drain should collect expired rumble state once before routing every request",
    );
    assert_source_order(
        routing_source,
        &[
            "ZrRuntimeHostRequestV1::Ime(request)",
            "if !ime_request_targets_viewport(&request, app.viewport)",
            "return;",
            "let Some(window) = app.window.as_ref()",
            "apply_runtime_ime_host_request(window.as_ref(), request)",
        ],
        "runtime host-request routing should apply an IME request only to its target viewport",
    );
    assert!(
        ime_mod_source.contains("pub(super) use request::apply_runtime_ime_host_request;"),
        "IME host-request module should expose only the request leaf entrypoint"
    );
    assert_source_order(
        ime_request_source,
        &[
            "ZrRuntimeImeHostRequestKindV1::SetCursorArea",
            "ime_logical_cursor_area(area)",
            ".with_cursor_area(position, size)",
        ],
        "IME cursor-area requests should validate before becoming unscaled winit logical updates",
    );
    assert!(
        ime_request_source.contains("runtime_ime_cursor_area_invalid"),
        "invalid IME cursor-area geometry must fail closed before the window API"
    );
    assert_source_order(
        ime_request_source,
        &[
            "ZrRuntimeImeHostRequestKindV1::SetSurroundingText",
            "runtime_ime_surrounding_text(text)",
            ".with_surrounding_text(text)",
        ],
        "IME surrounding-text requests should become winit surrounding-text updates",
    );
    assert!(
        ime_geometry_source.contains("LogicalPosition::new(area.x as f64, area.y as f64)"),
        "IME cursor-area x/y should stay in the geometry leaf"
    );
    assert!(
        ime_geometry_source.contains("LogicalSize::new(area.width as f64, area.height as f64)"),
        "IME cursor-area width/height should stay in the geometry leaf"
    );
    assert!(
        !ime_request_source.contains("scale_factor"),
        "IME cursor-area requests must submit the ABI's logical coordinates without DPI scaling"
    );
}

#[test]
fn runtime_entry_completes_clipboard_requests_on_the_target_window_thread() {
    let routing_source = include_str!("../../runtime_entry_app/host_requests/routing.rs");
    let clipboard_source = include_str!("../../runtime_entry_app/host_requests/clipboard/mod.rs");
    let windows_source =
        include_str!("../../runtime_entry_app/host_requests/clipboard/platform/windows.rs");

    assert!(
        routing_source.contains("ZrRuntimeHostRequestV1::Clipboard(request)"),
        "runtime host routing must retain the typed clipboard request variant"
    );
    assert_source_order(
        clipboard_source,
        &[
            "complete_clipboard_request(&request.request, &mut clipboard)",
            "ZrRuntimeClipboardResultV1::new(",
            "ZrRuntimeEventV1::clipboard_result(",
            "app.dispatch_runtime_event(event_loop, event)",
        ],
        "clipboard platform work must finish before its typed result returns to the runtime",
    );
    for required in [
        "window: app.window.as_deref()",
        "RawWindowHandle::Win32(window)",
        "ClipboardGuard::open(clipboard_owner(window)?)",
        "OpenClipboard(owner)",
        "GetClipboardData(u32::from(CF_UNICODETEXT))",
        "SetClipboardData(u32::from(CF_UNICODETEXT), handle)",
    ] {
        assert!(
            clipboard_source.contains(required) || windows_source.contains(required),
            "clipboard host bridge must preserve `{required}`"
        );
    }
    assert!(
        !windows_source.contains("OpenClipboard(ptr::null_mut())"),
        "Windows clipboard writes require the live target HWND as owner"
    );
}

#[test]
fn runtime_entry_keeps_ui_action_delivery_typed_and_diagnostics_content_free() {
    let routing_source = include_str!("../../runtime_entry_app/host_requests/routing.rs");
    let action_source = include_str!("../../runtime_entry_app/host_requests/ui_action.rs");

    assert_source_order(
        routing_source,
        &[
            "ZrRuntimeHostRequestV1::UiAction(request)",
            "report_unhandled_runtime_ui_action(app, request)",
        ],
        "runtime host routing must retain typed UI actions even without a product adapter",
    );
    for required in [
        "app.unhandled_ui_action_count.saturating_add(1)",
        "count.is_power_of_two()",
        "request.invocation.target_id()",
    ] {
        assert!(
            action_source.contains(required),
            "unhandled UI action diagnostics should retain bounded identity anchor `{required}`"
        );
    }
    assert!(
        !action_source.contains(concat!("request.invocation.", "payload")),
        "unhandled action diagnostics must never format template payload values"
    );
    assert!(
        !action_source.contains(concat!("request.", "secure_value")),
        "unhandled action diagnostics must never format opaque secure references"
    );
}

#[test]
fn runtime_entry_keeps_generic_ui_host_requests_typed_and_content_free() {
    let routing_source = include_str!("../../runtime_entry_app/host_requests/routing.rs");
    let request_source = include_str!("../../runtime_entry_app/host_requests/ui_host_request.rs");

    assert_source_order(
        routing_source,
        &[
            "ZrRuntimeHostRequestV1::UiHost(request)",
            "request.target_viewport != app.viewport",
            "report_unhandled_runtime_ui_host_request(app, request)",
        ],
        "runtime host routing must retain typed generic UI host requests for the target viewport",
    );
    for forbidden in [
        "request.kind.href",
        "request.kind.popup_id",
        "request.kind.tooltip_id",
    ] {
        assert!(
            !request_source.contains(forbidden),
            "generic UI host diagnostics must not format dynamic request content `{forbidden}`"
        );
    }
}
