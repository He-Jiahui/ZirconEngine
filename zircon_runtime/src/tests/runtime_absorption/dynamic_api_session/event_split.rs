#[test]
fn runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router() {
    let session_source = include_str!("../../../dynamic_api/session.rs");
    let events_source = include_str!("../../../dynamic_api/session/events.rs");
    let session_doc = include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_output = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md"
    );
    let runtime_index_output = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );

    for required_session_anchor in [
        "mod events;",
        "pub(super) unsafe fn handle_event(",
        "with_session(handle, |session| session.handle_event(event))",
    ] {
        assert!(
            session_source.contains(required_session_anchor),
            "session.rs should keep ABI entry owner anchor `{required_session_anchor}`"
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
            !session_source.contains(moved_event_anchor),
            "session.rs should not reclaim event owner helper `{moved_event_anchor}`"
        );
        assert!(
            events_source.contains(moved_event_anchor),
            "session/events.rs should own event helper `{moved_event_anchor}`"
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
