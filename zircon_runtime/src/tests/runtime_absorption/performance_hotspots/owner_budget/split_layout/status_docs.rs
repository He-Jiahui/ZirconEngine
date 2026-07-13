use super::super::{assert_contains_all, sources::OwnerBudgetSources};

const SLICE: &str = "Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split";
const STATUS: &str =
    "runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_split";

pub(super) fn assert_owner_budget_split_docs(sources: &OwnerBudgetSources) {
    for (label, source) in [("Runtime 07 numbered archive", sources.runtime_07_archive)] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                "owner_budget/split_layout",
                GUARD,
                "expected_test_file_count = 61",
            ],
        );
    }

    for (label, source) in [("Runtime 07 numbered archive", sources.runtime_07_archive)] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                "owner_budget/split_layout",
                "expected_test_file_count = 61",
            ],
        );
    }
}
