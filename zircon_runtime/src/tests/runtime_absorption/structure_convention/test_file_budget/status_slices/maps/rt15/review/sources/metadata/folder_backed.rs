use super::*;

#[test]
fn runtime_15_review_guard_source_metadata_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[4]);
    let children = read_review_root_sources(STRUCTURE_REVIEW_SOURCE_METADATA_CHILDREN);

    assert_contains_all(
        "review guard source metadata parent",
        &parent,
        &[
            "#[path = \"metadata/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"metadata/foundation_review.rs\"]",
            "mod foundation_review;",
            "#[path = \"metadata/root_expected_slice.rs\"]",
            "mod root_expected_slice;",
            "#[path = \"metadata/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"metadata/status_current.rs\"]",
            "mod status_current;",
            "#[path = \"metadata/structure_rows.rs\"]",
            "mod structure_rows;",
            "pub(in super::super) use foundation_review::*;",
            "pub(in super::super) use root_expected_slice::*;",
            "pub(in super::super) use source_inventory::*;",
            "pub(in super::super) use structure_rows::*;",
        ],
    );
    for moved_anchor in [
        "pub(in super::super) const ROUTE_SLICE",
        "pub(in super::super) const SOURCES_SLICE",
        "pub(in super::super) const REVIEW_FOUNDATION_MAPS_SLICE",
        "REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN: &[&str]",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "sources/metadata.rs should delegate moved metadata declaration {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard source metadata children",
        &children,
        &[
            ROUTE_SLICE,
            ROOT_GUARD_SLICE,
            ROOT_ROUTE_METADATA_GUARD_SLICE,
            SOURCES_SLICE,
            SOURCE_METADATA_GUARD_SLICE,
            REVIEW_GUARD_STRUCTURE_ROW_DATA_SLICE,
            REVIEW_FOUNDATION_MAPS_SLICE,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_SLICE,
            SOURCE_METADATA_GUARD_GUARD,
        ],
    );
    for path in STRUCTURE_REVIEW_SOURCE_METADATA_CHILDREN {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 95,
            "{path} should stay below the review-guard source metadata child budget; got {line_count} lines"
        );
    }
}
