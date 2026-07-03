use super::*;

#[test]
fn runtime_15_module_layout_child_summary_runtime_row_data_groups_are_child_owned() {
    let runtime_15_m4_row_data_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs",
    );
    let runtime_15_m4_row_data = format!(
        "{}\n{}",
        runtime_15_m4_row_data_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/row_ownership.rs",
        )
    );
    let runtime_15_m2_row_data_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data.rs",
    );
    let runtime_15_m2_row_data = format!(
        "{}\n{}",
        runtime_15_m2_row_data_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data/row_ownership.rs",
        )
    );
    let runtime_15_m3_row_data_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs",
    );
    let runtime_15_m3_row_data = format!(
        "{}\n{}",
        runtime_15_m3_row_data_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data/row_ownership.rs",
        )
    );

    assert_contains_all(
        "Runtime 15 M4 row-data child owns M4 split guard",
        &runtime_15_m4_row_data,
        &[
            "fn runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
            "Runtime 15 M4 status row child owns M4 row literals",
        ],
    );
    assert_contains_all(
        "Runtime 15 M2 row-data child owns M2 split guard",
        &runtime_15_m2_row_data,
        &[
            "fn runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
            "Runtime 15 M3 M2 row-data guard child-owner split",
            "runtime_15_m2_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 row-data child owns M3 split guard",
        &runtime_15_m3_row_data,
        &[
            "fn runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
            "Runtime 15 M3 status support child owns M3 row split literals",
        ],
    );
}
