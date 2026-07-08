use super::*;

#[test]
fn runtime_15_status_support_expected_slice_route_metadata_status_mirrors_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{ROUTE_METADATA_STATUS_MIRRORS_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(ROUTE_METADATA_STATUS_MIRRORS_CHILDREN);

    assert_contains_all(
        "status-support route metadata status-mirror parent",
        &parent,
        &[
            "#[path = \"mirrors/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mirrors/row_maps.rs\"]",
            "mod row_maps;",
            "#[path = \"mirrors/status_docs.rs\"]",
            "mod status_docs;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_status_support_expected_slice_rows",
        "read_repo(\"docs/",
        "ROUTE_METADATA_FRAMEWORKS_STATUS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status/route_meta/status_mirrors.rs should delegate moved status mirror {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-support route metadata status-mirror children",
        &children,
        &[
            "runtime_15_status_support_expected_slice_route_metadata_row_maps_are_registered",
            "runtime_15_status_support_expected_slice_route_metadata_docs_are_registered",
            ROUTE_METADATA_STATUS_MIRRORS_GUARD,
            ROUTE_METADATA_STATUS_MIRRORS_STATUS,
            ROUTE_METADATA_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ],
    );
}
