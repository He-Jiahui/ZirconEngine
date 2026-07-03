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
            "for request in requests",
            "apply_runtime_host_request(self, request);",
        ],
        "runtime entry host-request drain should route every drained runtime request",
    );
    assert_source_order(
        routing_source,
        &[
            "ZrRuntimeHostRequestV1::Ime(request)",
            "let Some(window) = app.window.as_ref()",
            "apply_runtime_ime_host_request(window.as_ref(), request)",
        ],
        "runtime host-request routing should send IME requests to the platform window leaf",
    );
    assert!(
        ime_mod_source.contains("pub(super) use request::apply_runtime_ime_host_request;"),
        "IME host-request module should expose only the request leaf entrypoint"
    );
    assert_source_order(
        ime_request_source,
        &[
            "ZrRuntimeImeHostRequestKindV1::SetCursorArea",
            "ImeRequest::Update(",
            ".with_cursor_area(ime_logical_position(area), ime_logical_size(area))",
        ],
        "IME cursor-area requests should become winit cursor-area updates",
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
}
