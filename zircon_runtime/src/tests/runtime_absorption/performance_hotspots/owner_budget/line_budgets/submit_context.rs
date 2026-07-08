use super::super::sources::OwnerBudgetSources;

pub(super) fn assert_submit_context_budgets(sources: &OwnerBudgetSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            sources.submit_context,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/camera_loop_sharing.rs",
            sources.submit_context_camera_loop,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/feedback_sidebands.rs",
            sources.submit_context_feedback_sidebands,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/source_extract_payloads.rs",
            sources.submit_context_source_extract_payloads,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/sources.rs",
            sources.submit_context_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/split_layout.rs",
            sources.submit_context_split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/status_docs.rs",
            sources.submit_context_status_docs,
        ),
    ] {
        super::assert_runtime_15_test_file_budget(path, source);
    }
}
