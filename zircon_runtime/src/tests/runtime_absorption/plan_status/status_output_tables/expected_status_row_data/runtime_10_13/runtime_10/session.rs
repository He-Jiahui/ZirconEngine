use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 10 Dynamic Session Event Split",
        [
            "session/events.rs",
            "runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router",
            "expected_source_file_count = 21",
            "active compile lanes deferred",
        ],
    ),
    (
        "Runtime 10 Dynamic Session Test Owner Split",
        [
            "session/tests/{mod,helpers,vampire_gameplay,vampire_menu,vampire_hud,frame_diagnostics,runtime_errors}.rs",
            "runtime_10_dynamic_session_test_owner_split_keeps_focused_modules",
            "session/tests/frame_diagnostics.rs",
            "standalone `dynamic_api_session.rs` 5/5",
        ],
    ),
];
