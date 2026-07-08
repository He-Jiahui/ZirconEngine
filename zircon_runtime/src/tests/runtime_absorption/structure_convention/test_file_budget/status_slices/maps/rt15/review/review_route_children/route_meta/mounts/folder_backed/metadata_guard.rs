use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_ROUTE);
    let children = format!(
        "{}\n{}\n{}",
        read_runtime_sources(STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_CHILDREN),
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN),
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_BUDGET_CHILDREN)
    );

    assert_contains_all(
        "review-route metadata parent mounts focused child owners",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"route_meta/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "STRUCTURE_REVIEW_ROUTE_PARENT",
        "STRUCTURE_REVIEW_ROUTE_CHILDREN",
        "read_status_support_expected_slice_rows",
        REVIEW_ROUTE_METADATA_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review_route_children/route_metadata.rs should delegate moved route metadata {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-route metadata children preserve moved guards",
        &children,
        &[
            REVIEW_ROUTE_METADATA_BUDGETS_GUARD,
            "runtime_15_review_guard_expected_slice_route_metadata_parent_routes_stay_budgeted",
            REVIEW_ROUTE_METADATA_GUARD,
            REVIEW_ROUTE_METADATA_GUARD_GUARD,
            "runtime_15_review_guard_expected_slice_route_metadata_status_is_mirrored",
        ],
    );
}
