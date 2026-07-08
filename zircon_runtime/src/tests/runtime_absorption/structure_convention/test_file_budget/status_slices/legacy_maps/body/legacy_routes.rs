use super::*;

#[test]
fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners() {
    let status_parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs",
    );
    let date_parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs",
    );
    let status_pre_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
    );
    let date_pre_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
    );
    let legacy_children = [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/expected_slice_guards.rs",
    );

    assert_contains_all(
        "expected-slice parents are routing-only after legacy split",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"status/pre_runtime_15.rs\"]",
            "#[path = \"date/pre_runtime_15.rs\"]",
            "pre_runtime_15::expected_status_for_slice(slice)",
            "pre_runtime_15::expected_date_for_slice(slice)",
        ],
    );
    for moved_literal in [
        "Runtime 14 Cargo 验证窗口探测",
        "Runtime 10 F18 asset manager resolution return shape",
        "Runtime 05 status-output expected anchor split",
        "Runtime 08 ECS root leaf owner guard",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status expected-slice parent should not keep pre-Runtime-15 literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date expected-slice parent should not keep pre-Runtime-15 literal {moved_literal}"
        );
    }

    assert_contains_all(
        "legacy status/date children own pre-Runtime-15 branches",
        &legacy_children,
        &[
            "Runtime 14 Cargo 验证窗口探测",
            "Runtime 10 F18 asset manager resolution return shape",
            "Runtime 05 status-output expected anchor split",
            "Runtime 08 ECS root leaf owner guard",
        ],
    );
    assert_contains_all(
        "legacy status/date parents route pre-Runtime-15 child groups",
        &format!("{status_pre_runtime_15}\n{date_pre_runtime_15}"),
        &[
            "runtime_01_05::expected_status_for_slice(slice)",
            "runtime_06_10::expected_status_for_slice(slice)",
            "runtime_11_14::expected_status_for_slice(slice)",
            "runtime_01_05::expected_date_for_slice(slice)",
            "runtime_06_10::expected_date_for_slice(slice)",
            "runtime_11_14::expected_date_for_slice(slice)",
            "mirror_docs_static_passed_cargo_pending",
            "2026-06-14",
        ],
    );
    assert_contains_all(
        "Runtime 15 status rows keep legacy child-owner split",
        &status_rows,
        &[
            "Runtime 15 M3 status output expected-slice legacy child-owner split",
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
            LEGACY_MAPS_GUARD,
        ],
    );
}
