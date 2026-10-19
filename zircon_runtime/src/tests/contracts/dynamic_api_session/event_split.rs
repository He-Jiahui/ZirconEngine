//! 动态会话的二进制接口、宿主请求和诊断路由需与共享契约同步。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router() {
    let session_source = include_str!("../../../dynamic_api/session.rs");
    let ffi_source = include_str!("../../../dynamic_api/session/ffi.rs");
    let state_source = include_str!("../../../dynamic_api/session/state.rs");
    let construction_source = include_str!("../../../dynamic_api/session/construction.rs");
    let events_source = include_str!("../../../dynamic_api/session/events.rs");
    let keyboard_source = include_str!("../../../dynamic_api/session/events/keyboard_ime.rs");
    let gamepad_source = include_str!("../../../dynamic_api/session/events/gamepad.rs");
    for (mount, route) in [
        (
            "#[path = \"events/keyboard_ime.rs\"]",
            "self.handle_keyboard(event)",
        ),
        (
            "#[path = \"events/gamepad.rs\"]",
            "self.handle_gamepad_axis(event)",
        ),
    ] {
        assert!(events_source.contains(mount) && events_source.contains(route));
    }
    let session_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/dynamic_api/session.md");
    let runtime_10_output = history("docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md");
    let runtime_index_output = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );

    assert!(session_source.contains("mod events;"));
    assert!(ffi_source.contains("pub(in crate::dynamic_api) unsafe fn handle_event("));
    let ffi_event = ffi_source
        .split("unsafe fn handle_event(")
        .nth(1)
        .and_then(|source| source.split("unsafe fn capture_frame(").next())
        .expect("FFI handle_event must retain its bounded ABI entry");
    for required_session_anchor in [
        ".checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)",
        "Ok(payload) => payload.to_vec()",
        "Err(error) if error.is_limit_exceeded()",
        "with_session_owned(handle, move |session|",
        "let event = ZrRuntimeEventV1 {",
        "data: payload.as_ptr()",
        "len: payload.len()",
        "session.handle_event(event)",
    ] {
        assert!(
            ffi_event.contains(required_session_anchor),
            "session/ffi.rs should keep ABI entry owner anchor `{required_session_anchor}`"
        );
    }

    assert!(
        ffi_event.find("payload.to_vec()").unwrap()
            < ffi_event.find("with_session_owned(handle,").unwrap()
    );
    assert!(
        ffi_event.find("data: payload.as_ptr()").unwrap()
            < ffi_event.find("session.handle_event(event)").unwrap()
    );

    for (source, moved_event_anchor) in [
        (events_source, "fn handle_mouse_button"),
        (events_source, "fn handle_mouse_wheel"),
        (keyboard_source, "fn handle_keyboard"),
        (keyboard_source, "fn handle_ime"),
        (gamepad_source, "fn handle_gamepad_axis"),
    ] {
        assert!(
            !ffi_source.contains(moved_event_anchor),
            "session/ffi.rs should not reclaim event owner helper `{moved_event_anchor}`"
        );
        assert!(
            source.contains(moved_event_anchor),
            "the mounted event owner should own event helper `{moved_event_anchor}`"
        );
    }

    for (owner, source) in [
        ("session/state.rs", state_source),
        ("session/construction.rs", construction_source),
        ("session/events.rs", events_source),
        ("session/events/keyboard_ime.rs", keyboard_source),
        ("session/events/gamepad.rs", gamepad_source),
    ] {
        for removed_editor_selection_anchor in [
            "selected_node",
            "selection_node",
            "selected_entity",
            "editor_selection",
            "set_selected_node",
            "sync_orbit_target_from_selection",
        ] {
            assert!(
                !source.contains(removed_editor_selection_anchor),
                "{owner} must not retain editor selection anchor `{removed_editor_selection_anchor}`"
            );
        }
    }

    for initial_orbit_anchor in [
        "matches!(&node.kind, NodeKind::Cube)",
        "node.transform.translation",
        "camera_controller.set_orbit_target(orbit_target)",
    ] {
        assert!(
            construction_source.contains(initial_orbit_anchor),
            "session/construction.rs must retain neutral initial orbit anchor `{initial_orbit_anchor}`"
        );
    }

    for required_events_anchor in [
        "pub(super) fn handle_event(&mut self, event: ZrRuntimeEventV1) -> ZrStatus",
        "UiAccessibilityActionRequest",
        "runtime_session_menu_action_at",
        "write_runtime_menu_action",
        "ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1",
    ] {
        assert!(
            events_source.contains(required_events_anchor),
            "session/events.rs should retain dispatch anchor `{required_events_anchor}`"
        );
    }

    for doc_anchor in [
        "Dynamic Session Event Split",
        "session/events.rs",
        "runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router",
        "expected_source_file_count = 35",
    ] {
        assert!(
            session_doc.contains(doc_anchor)
                || runtime_10_output.contains(doc_anchor)
                || runtime_index_output.contains(doc_anchor),
            "Runtime 10 event split docs should retain `{doc_anchor}`"
        );
    }
}
