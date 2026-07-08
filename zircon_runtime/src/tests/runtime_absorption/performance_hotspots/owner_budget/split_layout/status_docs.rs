use super::super::{assert_contains_all, sources::OwnerBudgetSources};

const SLICE: &str = "Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split";
const STATUS: &str =
    "runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_split";

pub(super) fn assert_owner_budget_split_docs(sources: &OwnerBudgetSources) {
    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan),
        ("Runtime index", sources.runtime_index),
        ("Runtime 07 plan", sources.runtime_07_plan),
        ("review findings", sources.review_findings),
        ("structure convention", sources.structure_convention),
        ("module convention doc", sources.module_doc),
        ("hotspot inventory doc", sources.hotspot_doc),
        ("status-output row data", sources.status_rows),
        ("session note", sources.session_note),
    ] {
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

    for (label, source) in [
        ("dynamic session doc", sources.dynamic_session_doc),
        ("ECS doc", sources.ecs_doc),
        ("runtime interface convergence doc", sources.interface_doc),
        ("runtime architecture review", sources.architecture_review),
    ] {
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

    assert_contains_all(
        "status-output status slice",
        sources.status_slice,
        &[SLICE, STATUS],
    );
    assert_contains_all(
        "status-output date slice",
        sources.date_slice,
        &[SLICE, "2026-07-06"],
    );
}
