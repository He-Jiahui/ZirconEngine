use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_child_sources_are_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN[0]);
    let children = format!(
        "{}\n{}",
        read_runtime_absorption_sources(REVIEW_ROUTE_CHILD_SOURCE_CHILDREN),
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN)
    );

    assert_contains_all(
        "review-route child sources route owner",
        &parent,
        &[
            "#[path = \"sources/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"sources/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"sources/helpers.rs\"]",
            "mod helpers;",
            "#[path = \"sources/route_metadata.rs\"]",
            "mod route_metadata;",
            "#[path = \"sources/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"sources/status_rows.rs\"]",
            "mod status_rows;",
            "pub(super) use guard_body::*;",
            "pub(super) use helpers::*;",
            "pub(super) use route_metadata::*;",
            "pub(super) use status_rows::*;",
        ],
    );
    for moved_anchor in [
        "const STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
        "REVIEW_ROUTE_GUARD_BODY_SLICE",
        "REVIEW_ROUTE_METADATA_SLICE",
        "STRUCTURE_REVIEW_CHILD_ROUTE_PARENT",
        "pub(super) fn read_status_support_expected_slice_rows",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review_route_children/sources.rs should delegate moved source anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-route child source children",
        &children,
        &[
            REVIEW_ROUTE_CHILD_SOURCES_GUARD,
            "REVIEW_ROUTE_GUARD_BODY_SLICE",
            "REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_SLICE",
            "read_status_support_expected_slice_rows",
            "read_runtime_absorption_sources",
        ],
    );
}
