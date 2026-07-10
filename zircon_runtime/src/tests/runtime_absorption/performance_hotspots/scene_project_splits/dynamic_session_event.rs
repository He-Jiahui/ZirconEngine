#[test]
fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner() {
    let session_root = include_str!("../../../../dynamic_api/session.rs");
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

    for root_anchor in [
        "mod events;",
        "pub(super) unsafe fn handle_event(",
        "with_session(handle, |session| session.handle_event(event))",
    ] {
        assert!(
            session_root.contains(root_anchor),
            "session.rs should keep dynamic ABI event entry anchor `{root_anchor}`"
        );
    }

    for moved_event_anchor in [
        "fn handle_mouse_button",
        "fn handle_mouse_wheel",
        "fn handle_keyboard",
        "fn handle_ime",
        "fn handle_gamepad_axis",
        "fn sync_orbit_target_from_selection",
    ] {
        assert!(
            !session_root.contains(moved_event_anchor),
            "session.rs should not reclaim dynamic event helper `{moved_event_anchor}`"
        );
        assert!(
            session_events.contains(moved_event_anchor),
            "session/events.rs should own dynamic event helper `{moved_event_anchor}`"
        );
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
