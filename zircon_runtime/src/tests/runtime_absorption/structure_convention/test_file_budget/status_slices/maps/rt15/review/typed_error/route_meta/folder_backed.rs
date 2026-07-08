use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_route_metadata_is_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{ROUTE_METADATA_ROUTE_PATH}"
    ));
    let children = read_route_metadata_children();

    assert_contains_all(
        "typed-error expected-slice route metadata parent",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"route_meta/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"route_meta/paths.rs\"]",
            "mod paths;",
            "#[path = \"route_meta/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"route_meta/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STATUS_REVIEW_TYPED_ERROR_CHILD",
        TYPED_ERROR_ROUTE_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed_error/route_metadata.rs should delegate moved route metadata anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "typed-error expected-slice route metadata children",
        &children,
        &[
            TYPED_ERROR_ROUTE_GUARD,
            ROUTE_METADATA_GUARD,
            "runtime_15_review_guard_expected_slice_typed_error_route_metadata_children_stay_budgeted",
            "runtime_15_review_guard_expected_slice_typed_error_route_metadata_docs_are_synced",
            "runtime_15_review_guard_expected_slice_typed_error_route_metadata_status_mirrors_are_synced",
        ],
    );
}
