use super::*;

#[test]
fn runtime_15_status_output_m3_row_data_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let runtime_15_m3 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );
    let foundation_guards = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );
    let status_support = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let ui_tests_second = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
    );
    let production_guard_support = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
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
        "top-level status rows include every Runtime 15 M3 child group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row parent exposes M3 child groups",
        &runtime_15,
        &[
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status row parent is a child-group aggregator",
        &runtime_15_m3,
        &[
            "#[path = \"m3/foundation_guards.rs\"]",
            "#[path = \"m3/status_support.rs\"]",
            "#[path = \"m3/ui_tests_second.rs\"]",
            "#[path = \"m3/production_guard_support.rs\"]",
            "pub(super) const FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 UI runtime input ownership test folder split",
        "Runtime 15 M3 status output Runtime 15 M3 row data split",
        "Runtime 15 M3 production file budget guard child-owner split",
    ] {
        assert!(
            !runtime_15_m3.contains(moved_row),
            "expected_status_row_data/runtime_15/m3.rs should delegate row literals instead of keeping {moved_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M3 child row owners keep representative row literals",
        &(foundation_guards.clone()
            + status_support.as_str()
            + ui_tests_second.as_str()
            + production_guard_support.as_str()),
        &[
            "Runtime 15 M3 graphics dead-code guard module split",
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
            "Runtime 15 M3 UI runtime input ownership test folder split",
            "Runtime 15 M3 status output M3 row data child-owner split",
            "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
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
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
            foundation_guards.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
            status_support.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
            ui_tests_second.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
            production_guard_support.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 expected status map records M3 child-owner split",
        &expected_status_map,
        &[
            "Runtime 15 M3 status output M3 row data child-owner split",
            "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records M3 child-owner split",
        &expected_date_map,
        &[
            "Runtime 15 M3 status output M3 row data child-owner split",
            "2026-06-24",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support row data",
            production_guard_support.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output M3 row data child-owner split",
                "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }
}
