use super::*;

#[test]
fn runtime_15_status_output_expected_slice_legacy_guard_body_children_stay_budgeted() {
    for (path, limit) in [
        (LEGACY_MAPS_PARENT, 20usize),
        (LEGACY_GUARD_BODY_PARENT, 25),
        (LEGACY_GUARD_BODY_CHILDREN[0], 70),
        (LEGACY_GUARD_BODY_CHILDREN[1], 95),
        (LEGACY_GUARD_BODY_CHILDREN[2], 120),
        (LEGACY_GUARD_BODY_CHILDREN[3], 35),
        (LEGACY_GUARD_BODY_CHILDREN[4], 115),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the Runtime 15 legacy guard-body budget {limit}; got {line_count} lines"
        );
    }

    for path in [
        "plan_status/status_output_tables/expected_slices/status.rs",
        "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
        "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
        "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
        "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
        "plan_status/status_output_tables/expected_slices/date.rs",
        "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
        "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
        "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
        "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
        "structure_convention/test_file_budget/status_output_expected_slices.rs",
    ] {
        let source = read_runtime_src(&format!("tests/runtime_absorption/{path}"));
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
