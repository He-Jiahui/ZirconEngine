use super::*;

#[test]
fn runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners() {
    let status_pre_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
    );
    let status_runtime_01_05 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
    );
    let status_runtime_06_10 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
    );
    let status_runtime_11_14 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
    );
    let date_pre_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
    );
    let date_runtime_01_05 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
    );
    let date_runtime_06_10 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
    );
    let date_runtime_11_14 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
    );
    let status_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "legacy expected-slice parents mount runtime group child owners",
        &format!("{status_pre_runtime_15}\n{date_pre_runtime_15}"),
        &[
            "#[path = \"pre_runtime_15/runtime_01_05.rs\"]",
            "mod runtime_01_05;",
            "#[path = \"pre_runtime_15/runtime_06_10.rs\"]",
            "mod runtime_06_10;",
            "#[path = \"pre_runtime_15/runtime_11_14.rs\"]",
            "mod runtime_11_14;",
        ],
    );
    for moved_literal in [
        "Runtime 05 plan-status Cargo attempt 状态审计",
        "Runtime 08 F17 entity path lookup verb rename",
        "Runtime 14 animation Cargo gate 尝试",
    ] {
        assert!(
            !status_pre_runtime_15.contains(moved_literal),
            "status/pre_runtime_15.rs should delegate grouped literal {moved_literal}"
        );
        assert!(
            !date_pre_runtime_15.contains(moved_literal),
            "date/pre_runtime_15.rs should delegate grouped literal {moved_literal}"
        );
    }
    assert_contains_all(
        "legacy expected-slice child groups own representative runtime ranges",
        &format!(
            "{status_runtime_01_05}\n{status_runtime_06_10}\n{status_runtime_11_14}\n{date_runtime_01_05}\n{date_runtime_06_10}\n{date_runtime_11_14}"
        ),
        &[
            "Runtime 05 plan-status Cargo attempt 状态审计",
            "Runtime 08 F17 entity path lookup verb rename",
            "Runtime 14 animation Cargo gate 尝试",
            "Some(\"cargo_attempt_status_static_passed_cargo_pending\")",
            "Some(\"runtime_08_entity_path_lookup_getter_rename_coremin_check_passed\")",
            "Some(\"cargo_blocked_external_compile_drift\")",
            "Some(\"2026-06-20\")",
            "Some(\"2026-06-22\")",
            "Some(\"2026-06-15\")",
        ],
    );
    assert_contains_all(
        "Runtime 15 status/date maps record legacy group split",
        &format!("{status_runtime_15}\n{date_runtime_15}"),
        &[
            "Runtime 15 M3 status output expected-slice legacy group child-owner split",
            "runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-24\")",
        ],
    );
    assert_contains_all(
        "Runtime 15 status rows record legacy group split",
        &status_rows,
        &[
            "Runtime 15 M3 status output expected-slice legacy group child-owner split",
            "runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
            "runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
            status_pre_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
            status_runtime_01_05.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
            status_runtime_06_10.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
            status_runtime_11_14.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
            date_pre_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
            date_runtime_01_05.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
            date_runtime_06_10.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
            date_runtime_11_14.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices.rs",
            include_str!("../status_output_expected_slices.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output expected-slice legacy group child-owner split",
                "runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
                "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
                "runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners",
            ],
        );
    }
}
