use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_route_mounts_are_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_ROUTE);
    let children = read_review_root_sources(STRUCTURE_REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN);

    assert_contains_all(
        "review guard root route-metadata route-mount parent",
        &parent,
        &[
            "#[path = \"mounts/child_owned.rs\"]",
            "mod child_owned;",
            "#[path = \"mounts/folder_backed.rs\"]",
            "mod folder_backed;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STRUCTURE_REVIEW_GUARD_PARENT",
        "STRUCTURE_REVIEW_ROUTE_CHILDREN",
        "STRUCTURE_REVIEW_GUARD_ROUTE_METADATA",
        ROUTE_GUARD,
        ROOT_ROUTE_METADATA_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review/route_meta/route_mounts.rs should delegate moved route-mount guard {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root route-metadata route-mount children preserve guards",
        &children,
        &[
            ROUTE_GUARD,
            ROOT_ROUTE_METADATA_GUARD,
            "runtime_15_review_guard_expected_slice_root_route_metadata_is_child_owned",
            "runtime_15_review_guard_expected_slice_root_route_metadata_guard_is_folder_backed",
        ],
    );
}

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_ROUTE_METADATA);
    let children = format!(
        "{}\n{}",
        read_review_root_sources(STRUCTURE_REVIEW_ROUTE_METADATA_CHILDREN),
        read_review_root_sources(STRUCTURE_REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN)
    );

    assert_contains_all(
        "review guard root route-metadata parent",
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
        "ROUTE_SLICE",
        "Cargo gate deferred",
        "let status_rows =",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review/route_metadata.rs should delegate moved route metadata {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root route-metadata children",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_root_route_metadata_sources_stay_budgeted",
            ROUTE_GUARD,
            ROOT_ROUTE_METADATA_GUARD,
        ],
    );
}
