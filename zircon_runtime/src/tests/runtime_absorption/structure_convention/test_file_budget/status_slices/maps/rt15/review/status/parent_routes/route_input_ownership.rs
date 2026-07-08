use super::*;

#[test]
fn runtime_15_status_support_parent_route_guard_route_inputs_are_child_owned() {
    let guard_parent = include_str!("../parent_route_children.rs");
    let route_inputs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/parent_routes/route_inputs.rs",
    );
    let source_reads = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/parent_routes/source_reads.rs",
    );

    assert_contains_all(
        "status-support parent route guard delegates route inputs",
        guard_parent,
        &[
            "#[path = \"parent_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"parent_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"parent_routes/source_reads.rs\"]",
            "mod source_reads;",
            "use route_inputs::*;",
            "use source_reads::*;",
        ],
    );

    for parent_owned_literal in [
        "const STATUS_SUPPORT_PARENT_ROUTE_CHILDREN: &[&str]",
        "const DATE_SUPPORT_PARENT_ROUTE_CHILDREN: &[&str]",
        "const STRUCTURE_REVIEW_STATUS_SUPPORT_PARENT_ROUTE_GUARD_CHILDREN: &[&str]",
        "fn read_sources(paths: &[&str]) -> String",
    ] {
        assert!(
            !guard_parent.contains(parent_owned_literal),
            "status-support parent route guard should delegate {parent_owned_literal}"
        );
    }

    assert_contains_all(
        "status-support parent route input owner keeps route lists",
        &route_inputs,
        &[
            "STATUS_SUPPORT_PARENT_ROUTE_CHILDREN",
            "DATE_SUPPORT_PARENT_ROUTE_CHILDREN",
            "STRUCTURE_REVIEW_STATUS_SUPPORT_PARENT_ROUTE_GUARD_CHILDREN",
            "status_support_maps/runtime_row_data_maps.rs",
            "status_support_maps/m3_m4_expected_slice_maps.rs",
            "parent_routes/route_input_ownership.rs",
            "parent_routes/source_reads.rs",
        ],
    );

    assert_contains_all(
        "status-support parent route source helper keeps nested map expansion",
        &source_reads,
        &[
            "pub(super) fn read_sources(paths: &[&str]) -> String",
            "path.ends_with(\"status_support_maps/m3_m4_expected_slice_maps.rs\")",
            "\"expected_slice_guard_maps\"",
            "\"status_support_guard_maps\"",
        ],
    );
}
