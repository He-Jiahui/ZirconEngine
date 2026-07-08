use super::*;

#[test]
fn runtime_15_review_guard_root_sources_are_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_SOURCES);
    let children = format!(
        "{}\n{}\n{}",
        read_review_root_sources(STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN),
        read_review_root_sources(STRUCTURE_REVIEW_SOURCE_METADATA_CHILDREN),
        read_review_structure_path_sources(STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN)
    );

    assert_contains_all(
        "review guard root sources route owner",
        &parent,
        &[
            "#[path = \"sources/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"sources/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"sources/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"sources/foundation_review_maps.rs\"]",
            "mod foundation_review_maps;",
            "#[path = \"sources/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"sources/route_children.rs\"]",
            "mod route_children;",
            "#[path = \"sources/status_maps.rs\"]",
            "mod status_maps;",
            "#[path = \"sources/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"sources/structure_paths.rs\"]",
            "mod structure_paths;",
            "pub(super) use metadata::*;",
            "pub(super) use route_children::*;",
            "pub(super) use status_maps::*;",
            "pub(super) use structure_paths::*;",
        ],
    );
    for moved_anchor in [
        "const STATUS_REVIEW_FOUNDATION_CHILD",
        "STRUCTURE_REVIEW_ROUTE_CHILDREN",
        "read_review_root_sources",
        "STRUCTURE_SUPPORT_ROWS",
        "ROOT_STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
        SOURCE_METADATA_GUARD_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review/sources.rs should delegate moved source inventory {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root sources children",
        &children,
        &[
            ROUTE_SLICE,
            ROOT_GUARD_GUARD,
            ROOT_ROUTE_METADATA_GUARD,
            SOURCES_SLICE,
            SOURCES_GUARD,
            REVIEW_FOUNDATION_MAPS_GUARD,
            SOURCE_METADATA_GUARD_GUARD,
            "STATUS_REVIEW_FOUNDATION_CHILD",
            "STRUCTURE_REVIEW_ROUTE_CHILDREN",
            "read_review_root_sources",
            "runtime_15_review_guard_source_inventory_sources_stay_budgeted",
            "runtime_15_review_guard_source_inventory_status_is_mirrored",
            "runtime_15_review_guard_source_inventory_docs_are_synced",
        ],
    );
}
