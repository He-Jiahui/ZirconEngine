use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_PARENT);
    let children = read_review_root_sources(STRUCTURE_REVIEW_ROUTE_CHILDREN);
    let guard_body_children = read_review_root_sources(STRUCTURE_REVIEW_GUARD_BODY_CHILDREN);

    assert_contains_all(
        "review_guard_maps.rs root route",
        &parent,
        &[
            "#[path = \"review/sources.rs\"]",
            "mod sources;",
            "#[path = \"review/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"review/route_metadata.rs\"]",
            "mod route_metadata;",
            "#[path = \"review/review_route_children.rs\"]",
            "mod review_route_children;",
            "use sources::*;",
        ],
    );
    for moved_anchor in [
        "const STATUS_REVIEW_FOUNDATION_CHILD",
        "#[test]",
        "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review_guard_maps.rs should delegate moved root route metadata {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root route children",
        &format!("{children}\n{guard_body_children}"),
        &[
            "STATUS_REVIEW_FOUNDATION_CHILD",
            "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
            ROUTE_GUARD,
            ROOT_GUARD_GUARD,
        ],
    );
}
