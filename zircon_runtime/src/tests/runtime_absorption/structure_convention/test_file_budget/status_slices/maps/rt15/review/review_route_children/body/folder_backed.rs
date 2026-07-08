use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_guard_body_is_child_owned() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{REVIEW_ROUTE_GUARD_BODY_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(REVIEW_ROUTE_GUARD_BODY_CHILDREN);

    assert_contains_all(
        "review-route guard body parent mounts child checks",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/guard_status.rs\"]",
            "mod guard_status;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STATUS_REVIEW_FOUNDATION_CHILD",
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        "runtime_15_review_guard_expected_slice_maps_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review_route_children/guard_body.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-route guard body children own moved checks",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_maps_are_folder_backed",
            "runtime_15_review_guard_expected_slice_sources_stay_budgeted",
            "runtime_15_review_guard_expected_slice_guard_body_status_mirrors_are_registered",
            "runtime_15_review_guard_expected_slice_status_mirrors_are_registered",
            REVIEW_ROUTE_GUARD_BODY_GUARD,
        ],
    );
}
