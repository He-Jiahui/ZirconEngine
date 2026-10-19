//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner() {
    let session_facade = include_str!("../../../dynamic_api/session.rs");
    let session_ffi = include_str!("../../../dynamic_api/session/ffi.rs");
    let session_state = include_str!("../../../dynamic_api/session/state.rs");
    let session_construction = include_str!("../../../dynamic_api/session/construction.rs");
    let session_events = include_str!("../../../dynamic_api/session/events.rs");
    let keyboard_ime = include_str!("../../../dynamic_api/session/events/keyboard_ime.rs");
    let gamepad = include_str!("../../../dynamic_api/session/events/gamepad.rs");
    let runtime_07_output = history("docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md");
    let runtime_index_output =
        history("docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-index-output-records.md");
    let hotspot_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/dynamic_api/session.md");

    assert!(
        session_facade.contains("mod events;"),
        "session.rs should keep the event owner module declaration"
    );
    for ffi_anchor in [
        "pub(in crate::dynamic_api) unsafe fn handle_event(",
        ".checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)",
        "Ok(payload) => payload.to_vec()",
        "with_session_owned(handle, move |session| {",
        "data: payload.as_ptr()",
        "len: payload.len()",
        "event.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1",
        "session.handle_event(event)",
    ] {
        assert!(
            session_ffi.contains(ffi_anchor),
            "session/ffi.rs should keep dynamic ABI event entry anchor `{ffi_anchor}`"
        );
    }

    for (moved_event_anchor, owner, source) in [
        (
            "fn handle_mouse_button",
            "session/events.rs",
            session_events,
        ),
        ("fn handle_mouse_wheel", "session/events.rs", session_events),
        ("fn handle_keyboard", "events/keyboard_ime.rs", keyboard_ime),
        ("fn handle_ime", "events/keyboard_ime.rs", keyboard_ime),
        ("fn handle_gamepad_axis", "events/gamepad.rs", gamepad),
    ] {
        assert!(
            !session_facade.contains(moved_event_anchor)
                && !session_ffi.contains(moved_event_anchor),
            "session facade and FFI owner should not reclaim dynamic event helper `{moved_event_anchor}`"
        );
        assert!(
            source.contains(moved_event_anchor),
            "{owner} should own dynamic event helper `{moved_event_anchor}`"
        );
    }

    for (owner, source) in [
        ("session/state.rs", session_state),
        ("session/construction.rs", session_construction),
        ("session/events.rs", session_events),
        ("events/keyboard_ime.rs", keyboard_ime),
        ("events/gamepad.rs", gamepad),
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

    for events_anchor in [
        "#[path = \"events/keyboard_ime.rs\"]",
        "#[path = \"events/gamepad.rs\"]",
        "self.handle_keyboard(event)",
        "self.handle_ime(event)",
        "self.handle_gamepad_axis(event)",
        "pub(super) fn handle_event(&mut self, event: ZrRuntimeEventV1) -> ZrStatus",
        "UiAccessibilityActionRequest",
        "runtime_session_menu_action_at",
        "write_runtime_menu_action",
        "ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1",
    ] {
        assert!(
            session_events.contains(events_anchor),
            "session/events.rs should retain dynamic event dispatch anchor `{events_anchor}`"
        );
    }

    for doc_anchor in [
        "Dynamic Session Event Split",
        "session/events.rs",
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
        "dynamic session event split",
    ] {
        assert!(
            runtime_07_output.contains(doc_anchor)
                || runtime_index_output.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || dynamic_session_doc.contains(doc_anchor),
            "Dynamic session event split docs should retain `{doc_anchor}`"
        );
    }
}
