use super::*;

#[test]
fn runtime_15_status_output_runtime_15_m3_row_data_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let runtime_15_m3 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );
    let runtime_15_m4 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "top-level status row data aggregation keeps Runtime 15 M3 group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row parent delegates M3 rows",
        &runtime_15,
        &[
            "#[path = \"runtime_15/m3.rs\"]",
            "mod m3;",
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "#[path = \"runtime_15/m4.rs\"]",
            "mod m4;",
        ],
    );
    for moved_m3_row in [
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 status output Runtime 15 row data split",
        "Runtime 15 M3 status output expected-slice maps split",
        "runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !runtime_15.contains(moved_m3_row),
            "expected_status_row_data/runtime_15.rs should delegate M3 row literals instead of keeping {moved_m3_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M3 status row parent mounts M3 child groups",
        &runtime_15_m3,
        &[
            "#[path = \"m3/foundation_guards.rs\"]",
            "mod foundation_guards;",
            "#[path = \"m3/status_support.rs\"]",
            "mod status_support;",
            "pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status support child owns M3 row split literals",
        &runtime_15_m3_status_support,
        &[
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 M3 status output Runtime 15 row data split",
            "Runtime 15 M3 status output Runtime 15 M4 row data split",
            "Runtime 15 M3 status output expected-slice maps split",
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
            "runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred",
            "runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_status_row_data.rs",
            parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
            runtime_15_m3.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
            runtime_15_m4.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
            runtime_15_m3_status_support.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data.rs",
            include_str!("status_output_row_data.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 expected status map records M3 row split",
        &expected_status_map,
        &[
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
            "runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records M3 row split",
        &expected_date_map,
        &[
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
            "2026-06-23",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 row data",
            runtime_15_m3_status_support.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output Runtime 15 M3 row data split",
                "runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
            ],
        );
    }
}
