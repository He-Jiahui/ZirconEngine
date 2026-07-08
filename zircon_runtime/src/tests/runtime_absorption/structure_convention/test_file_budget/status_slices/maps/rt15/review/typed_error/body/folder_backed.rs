use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_guard_body_is_folder_backed() {
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{GUARD_BODY_ROUTE_PATH}"));
    let children = read_guard_body_children();

    assert_contains_all(
        "typed-error expected-slice guard-body parent",
        &parent,
        &[
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"body/paths.rs\"]",
            "mod paths;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STATUS_REVIEW_TYPED_ERROR_CHILD",
        "Runtime 15 M3 typed-error convergence guard child-owner split",
        "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed_error/guard_body.rs should delegate moved guard-body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "typed-error expected-slice guard-body children",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
            "runtime_15_review_guard_expected_slice_typed_error_literals_are_child_owned",
            "runtime_15_review_guard_expected_slice_typed_error_guard_body_status_is_synced",
            GUARD_BODY_GUARD,
            GUARD_BODY_SLICE,
            GUARD_BODY_STATUS,
        ],
    );
}
