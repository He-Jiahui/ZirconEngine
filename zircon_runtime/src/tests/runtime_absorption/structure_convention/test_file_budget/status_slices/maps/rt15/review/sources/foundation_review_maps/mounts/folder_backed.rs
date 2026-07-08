use super::*;

#[test]
fn runtime_15_review_guard_foundation_route_mounts_are_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_FOUNDATION_MAPS_CHILDREN[0]);
    let children = read_review_root_sources(STRUCTURE_REVIEW_FOUNDATION_ROUTE_MOUNT_CHILDREN);

    assert_contains_all(
        "review foundation route-mount parent",
        &parent,
        &[
            "#[path = \"mounts/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mounts/status_date_routes.rs\"]",
            "mod status_date_routes;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STATUS_REVIEW_FOUNDATION_CHILD",
        "REVIEW_FOUNDATION_MAPS_STATUS",
        "REVIEW_FOUNDATION_MAP_GUARD_GUARD",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "foundation_review_maps/route_mounts.rs should delegate moved route guard {moved_anchor}"
        );
    }
    assert_contains_all(
        "review foundation route-mount children",
        &children,
        &[
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_GUARD,
            REVIEW_FOUNDATION_MAP_GUARD_GUARD,
            "runtime_15_review_guard_foundation_status_date_maps_are_folder_backed",
        ],
    );
}

#[test]
fn runtime_15_review_guard_foundation_status_date_map_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[3]);
    let children = format!(
        "{}\n{}",
        read_review_root_sources(STRUCTURE_REVIEW_FOUNDATION_MAPS_CHILDREN),
        read_review_root_sources(STRUCTURE_REVIEW_FOUNDATION_ROUTE_MOUNT_CHILDREN)
    );

    assert_contains_all(
        "review foundation map guard parent",
        &parent,
        &[
            "#[path = \"foundation_review_maps/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"foundation_review_maps/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_repo(",
        "read_status_review_foundation_sources",
        "REVIEW_FOUNDATION_MAPS_STATUS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "foundation_review_maps.rs should delegate moved guard source {moved_anchor}"
        );
    }
    assert_contains_all(
        "review foundation map guard children",
        &children,
        &[
            "runtime_15_review_guard_foundation_status_date_maps_are_folder_backed",
            "runtime_15_review_guard_foundation_status_date_maps_status_is_mirrored",
            REVIEW_FOUNDATION_MAP_GUARD_GUARD,
        ],
    );
}
