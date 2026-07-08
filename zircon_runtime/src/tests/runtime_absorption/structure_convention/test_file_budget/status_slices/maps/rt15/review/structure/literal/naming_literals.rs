use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_naming_literals_are_child_owned() {
    let status_naming_child = read_runtime_src(STATUS_NAMING_CHILD);
    let date_naming_child = read_runtime_src(DATE_NAMING_CHILD);

    assert_contains_all(
        "naming expected-slice children own naming guard literals",
        &format!("{status_naming_child}\n{date_naming_child}"),
        &[
            "Runtime 15 M3 graphics GPU-model guard child-owner split",
            "runtime_15_banned_names_global_module_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 plugin static manifest naming guard child-owner split",
            "Some(\"2026-06-30\")",
        ],
    );
}
