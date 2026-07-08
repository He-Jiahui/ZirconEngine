use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_budgets_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{}",
        REVIEW_ROUTE_METADATA_BUDGETS_ROUTE_PATH
    ));
    let children = read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_BUDGET_CHILDREN);

    assert_contains_all(
        "review-route route-metadata budgets parent mounts child modules",
        &parent,
        &[
            "#[path = \"budgets/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"budgets/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"budgets/parent_routes.rs\"]",
            "mod parent_routes;",
            "#[path = \"budgets/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"budgets/source_children.rs\"]",
            "mod source_children;",
            "#[path = \"budgets/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"budgets/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    for moved_anchor in [
        "fn runtime_15_review_guard_expected_slice_route_metadata_sources_stay_budgeted",
        "STRUCTURE_REVIEW_CHILD_ROUTE_PARENT",
        "REVIEW_ROUTE_CHILD_SOURCE_CHILDREN",
        "REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN",
        "REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN",
        "REVIEW_ROUTE_GUARD_BODY_CHILDREN",
        "read_status_support_expected_slice_rows",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review-route route-metadata budgets parent should delegate {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-route route-metadata budgets children own moved checks",
        &children,
        &[
            REVIEW_ROUTE_METADATA_BUDGETS_GUARD,
            "runtime_15_review_guard_expected_slice_route_metadata_parent_routes_stay_budgeted",
            "runtime_15_review_guard_expected_slice_route_metadata_source_children_stay_budgeted",
            "runtime_15_review_guard_expected_slice_route_metadata_route_mounts_stay_budgeted",
            "runtime_15_review_guard_expected_slice_route_metadata_status_mirrors_stay_budgeted",
            "runtime_15_review_guard_expected_slice_route_metadata_guard_body_children_stay_budgeted",
            "read_status_support_expected_slice_rows",
        ],
    );

    for (path, limit) in [
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[0], 80usize),
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[1], 35),
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[2], 30),
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[3], 55),
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[4], 30),
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[5], 95),
        (REVIEW_ROUTE_METADATA_BUDGET_CHILDREN[6], 30),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route route-metadata budgets child budget {limit}; got {line_count} lines"
        );
    }
}
