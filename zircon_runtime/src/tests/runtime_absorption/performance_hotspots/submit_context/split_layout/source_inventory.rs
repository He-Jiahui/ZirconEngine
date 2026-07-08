use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_submit_context_source_inventory(sources: &SplitLayoutSources) {
    assert_contains_all(
        "performance hotpath source inventory",
        sources.source_inventory,
        &[
            "EXPECTED_TEST_FILE_COUNT = 91",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/camera_loop_sharing.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/feedback_sidebands.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/source_extract_payloads.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/sources.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/split_layout.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/split_layout/route.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/split_layout/source_inventory.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/split_layout/sources.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/split_layout/status_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/status_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
        ],
    );
}
