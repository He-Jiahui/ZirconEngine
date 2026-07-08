use super::super::sources::OwnerBudgetSources;

pub(super) fn assert_root_file_budgets(sources: &OwnerBudgetSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots.rs",
            sources.performance_parent,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_error_paths.rs",
            sources.submit_error_paths,
        ),
    ] {
        super::assert_runtime_15_test_file_budget(path, source);
    }
}
