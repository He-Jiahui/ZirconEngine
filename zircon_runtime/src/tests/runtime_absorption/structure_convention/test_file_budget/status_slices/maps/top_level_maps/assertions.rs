use super::{sources::TopLevelMapSources, *};

#[path = "assertions/line_budgets.rs"]
mod line_budgets;
#[path = "assertions/pre_runtime_15_maps.rs"]
mod pre_runtime_15_maps;
#[path = "assertions/runtime_15_maps.rs"]
mod runtime_15_maps;
#[path = "assertions/status_and_docs.rs"]
mod status_and_docs;

pub(super) fn assert_expected_slice_maps_are_child_owners(sources: &TopLevelMapSources) {
    assert_contains_all(
        "status expected-slice parent delegates Runtime 15",
        &sources.status_parent,
        &[
            "#[path = \"status/pre_runtime_15.rs\"]",
            "mod pre_runtime_15;",
            "#[path = \"status/runtime_15.rs\"]",
            "mod runtime_15;",
            "runtime_15::expected_status_for_slice(slice)",
            "pre_runtime_15::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "date expected-slice parent delegates Runtime 15",
        &sources.date_parent,
        &[
            "#[path = \"date/pre_runtime_15.rs\"]",
            "mod pre_runtime_15;",
            "#[path = \"date/runtime_15.rs\"]",
            "mod runtime_15;",
            "runtime_15::expected_date_for_slice(slice)",
            "pre_runtime_15::expected_date_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "test file budget root mounts expected-slice guard",
        &sources.test_budget_parent,
        &["mod status_output_expected_slices;"],
    );

    for moved_runtime_15_slice in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "Runtime 15 M3 status output Runtime 15 M4 row data split",
        "runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !sources.status_parent.contains(moved_runtime_15_slice),
            "expected_slices/status.rs should delegate Runtime 15 status literals instead of keeping {moved_runtime_15_slice}"
        );
        assert!(
            !sources.date_parent.contains(moved_runtime_15_slice),
            "expected_slices/date.rs should delegate Runtime 15 date literals instead of keeping {moved_runtime_15_slice}"
        );
    }
    for moved_pre_runtime_15_slice in [
        "Runtime 14 Cargo 验证窗口探测",
        "Runtime 05 plan-status Cargo attempt 状态审计",
        "Runtime 11 full-lib default after graphics exposure retry",
        "Runtime 12 input boundary grouped manager import guard repair",
    ] {
        assert!(
            !sources.status_parent.contains(moved_pre_runtime_15_slice),
            "expected_slices/status.rs should delegate pre-Runtime-15 status literals instead of keeping {moved_pre_runtime_15_slice}"
        );
        assert!(
            !sources.date_parent.contains(moved_pre_runtime_15_slice),
            "expected_slices/date.rs should delegate pre-Runtime-15 date literals instead of keeping {moved_pre_runtime_15_slice}"
        );
    }

    runtime_15_maps::assert_runtime_15_maps(sources);
    pre_runtime_15_maps::assert_pre_runtime_15_maps(sources);
    line_budgets::assert_line_budgets(sources);
    status_and_docs::assert_status_and_docs(sources);
}
