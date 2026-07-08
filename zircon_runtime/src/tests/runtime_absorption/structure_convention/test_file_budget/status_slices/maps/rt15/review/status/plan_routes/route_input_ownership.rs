use super::*;

#[test]
fn runtime_15_status_support_plan_doc_route_guard_route_inputs_are_child_owned() {
    let guard_parent = include_str!("../plan_doc_route_children.rs");
    let route_inputs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/plan_routes/route_inputs.rs",
    );
    let child_paths = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/plan_routes/child_paths.rs",
    );
    let source_reads = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/plan_routes/source_reads.rs",
    );

    assert_contains_all(
        "status-support plan-doc route guard delegates route inputs",
        guard_parent,
        &[
            "#[path = \"plan_routes/child_paths.rs\"]",
            "mod child_paths;",
            "#[path = \"plan_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"plan_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"plan_routes/source_reads.rs\"]",
            "mod source_reads;",
            "use child_paths::*;",
            "use route_inputs::*;",
            "use source_reads::*;",
        ],
    );

    for parent_owned_literal in [
        "const STRUCTURE_REVIEW_STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILD: &str",
        "const STRUCTURE_REVIEW_STATUS_SUPPORT_PLAN_DOC_ROUTE_GUARD_CHILDREN: &[&str]",
        "fn status_support_plan_doc_child_paths() -> Vec<&'static str>",
        "fn date_support_plan_doc_child_paths() -> Vec<&'static str>",
        "fn read_sources(paths: &[&str]) -> Vec<String>",
    ] {
        assert!(
            !guard_parent.contains(parent_owned_literal),
            "status-support plan-doc route guard should delegate {parent_owned_literal}"
        );
    }

    assert_contains_all(
        "status-support plan-doc route input owner keeps guard path lists",
        &route_inputs,
        &[
            "STRUCTURE_REVIEW_STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILD",
            "STRUCTURE_REVIEW_STATUS_SUPPORT_PLAN_DOC_ROUTE_GUARD_CHILDREN",
            "plan_routes/route_input_ownership.rs",
            "plan_routes/child_paths.rs",
            "plan_routes/source_reads.rs",
        ],
    );

    assert_contains_all(
        "status-support plan-doc child path owner keeps route aggregation",
        &child_paths,
        &[
            "pub(super) fn status_support_plan_doc_child_paths()",
            "STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN.to_vec()",
            "STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN",
            "STATUS_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN",
            "pub(super) fn date_support_plan_doc_child_paths()",
        ],
    );

    assert_contains_all(
        "status-support plan-doc route source helper stays child-owned",
        &source_reads,
        &["pub(super) fn read_sources(paths: &[&str]) -> Vec<String>"],
    );
}
