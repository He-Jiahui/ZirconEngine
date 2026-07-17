#[test]
fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner() {
    let session_facade = include_str!("../../../../dynamic_api/session.rs");
    let session_ffi = include_str!("../../../../dynamic_api/session/ffi.rs");
    let session_state = include_str!("../../../../dynamic_api/session/state.rs");
    let session_construction = include_str!("../../../../dynamic_api/session/construction.rs");
    let session_events = include_str!("../../../../dynamic_api/session/events.rs");
    let runtime_07_output = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
    );
    let runtime_index_output = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-index-output-records.md"
    );
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../../docs/zircon_runtime/dynamic_api/session.md");

    assert!(
        session_facade.contains("mod events;"),
        "session.rs should keep the event owner module declaration"
    );
    for ffi_anchor in [
        "pub(in crate::dynamic_api) unsafe fn handle_event(",
        "with_session(handle, |session| session.handle_event(event))",
    ] {
        assert!(
            session_ffi.contains(ffi_anchor),
            "session/ffi.rs should keep dynamic ABI event entry anchor `{ffi_anchor}`"
        );
    }

    for moved_event_anchor in [
        "fn handle_mouse_button",
        "fn handle_mouse_wheel",
        "fn handle_keyboard",
        "fn handle_ime",
        "fn handle_gamepad_axis",
    ] {
        assert!(
            !session_facade.contains(moved_event_anchor)
                && !session_ffi.contains(moved_event_anchor),
            "session facade and FFI owner should not reclaim dynamic event helper `{moved_event_anchor}`"
        );
        assert!(
            session_events.contains(moved_event_anchor),
            "session/events.rs should own dynamic event helper `{moved_event_anchor}`"
        );
    }

    for (owner, source) in [
        ("session/state.rs", session_state),
        ("session/construction.rs", session_construction),
        ("session/events.rs", session_events),
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
