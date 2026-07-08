use super::*;

#[test]
fn runtime_15_status_support_review_guard_row_data_route_guard_route_inputs_are_child_owned() {
    let guard_parent = include_str!("../review_guard_row_data_route_children.rs");
    let route_inputs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/route_inputs.rs",
    );
    let child_paths = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/child_paths.rs",
    );
    let source_reads = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/source_reads.rs",
    );

    assert_contains_all(
        "status-support review-guard row-data route guard delegates route inputs",
        guard_parent,
        &[
            "#[path = \"review_row_routes/child_paths.rs\"]",
            "mod child_paths;",
            "#[path = \"review_row_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"review_row_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"review_row_routes/source_reads.rs\"]",
            "mod source_reads;",
            "use child_paths::*;",
            "use route_inputs::*;",
            "use source_reads::*;",
        ],
    );

    for parent_owned_literal in [
        "const STRUCTURE_REVIEW_STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILD: &str",
        "const STRUCTURE_REVIEW_STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_GUARD_CHILDREN: &[&str]",
        "const STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD: &str",
        "const DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD: &str",
        "fn status_support_review_guard_row_data_child_paths() -> Vec<&'static str>",
        "fn date_support_review_guard_row_data_child_paths() -> Vec<&'static str>",
        "fn read_sources(paths: &[&str]) -> Vec<String>",
    ] {
        assert!(
            !guard_parent.contains(parent_owned_literal),
            "status-support review-guard row-data route guard should delegate {parent_owned_literal}"
        );
    }

    assert_contains_all(
        "status-support review-guard row-data route input owner keeps guard path lists",
        &route_inputs,
        &[
            "STRUCTURE_REVIEW_STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILD",
            "STRUCTURE_REVIEW_STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_GUARD_CHILDREN",
            "STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD",
            "DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD",
            "review_row_routes/route_input_ownership.rs",
            "review_row_routes/child_paths.rs",
            "review_row_routes/source_reads.rs",
        ],
    );

    assert_contains_all(
        "status-support review-guard row-data child path owner keeps route aggregation",
        &child_paths,
        &[
            "pub(super) fn status_support_review_guard_row_data_child_paths()",
            "STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN.to_vec()",
            "pub(super) fn date_support_review_guard_row_data_child_paths()",
            "DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN.to_vec()",
        ],
    );

    assert_contains_all(
        "status-support review-guard row-data route source helper stays child-owned",
        &source_reads,
        &["pub(super) fn read_sources(paths: &[&str]) -> Vec<String>"],
    );
}
