use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_guard_is_folder_backed() {
    let parent = read_runtime_src(FOUNDATION_ROUTE_PARENT);
    let children = FOUNDATION_GUARD_CHILDREN
        .iter()
        .chain(FOUNDATION_STATUS_MIRRORS_CHILDREN.iter())
        .map(|path| read_runtime_absorption_child(path))
        .collect::<Vec<_>>()
        .join("\n");

    for moved_anchor in [
        "#[test]",
        "let status_parent = read_runtime_src(STATUS_PARENT)",
        "Runtime 15 foundation expected-slice map row data",
        "read_plan_status_child_sources(",
        FOUNDATION_MAP_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "rt15_slices/foundation.rs should delegate moved guard anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "foundation expected-slice map guard children keep moved checks",
        &children,
        &[
            "runtime_15_foundation_expected_slice_maps_route_mounts_are_child_owned",
            "runtime_15_foundation_expected_slice_maps_are_folder_backed",
            "runtime_15_foundation_expected_slice_maps_status_mirrors_are_synced",
            "runtime_15_foundation_expected_slice_maps_docs_are_synced",
            "runtime_15_foundation_expected_slice_maps_status_mirrors_are_folder_backed",
            "runtime_15_foundation_expected_slice_maps_guard_children_stay_budgeted",
            FOUNDATION_GUARD,
        ],
    );
}
