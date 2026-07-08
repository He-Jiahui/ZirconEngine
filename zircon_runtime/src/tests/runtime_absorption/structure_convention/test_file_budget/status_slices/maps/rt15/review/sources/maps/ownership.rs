use super::*;
#[test]
fn runtime_15_review_guard_source_status_maps_are_child_owned() {
    let parent = include_str!("../status_maps.rs");
    let child_sources = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[1]);
    let foundation_routes = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[3]);
    let parent_routes = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[5]);
    let source_reads = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[6]);
    let status_metadata = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[7]);
    let support_routes = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[9]);
    let typed_error_routes = read_status_map_child(SOURCE_STATUS_MAPS_CHILDREN[10]);
    assert_contains_all(
        "review guard source status-map parent delegates child owners",
        parent,
        &[
            "#[path = \"maps/budgets.rs\"]",
            "#[path = \"maps/child_sources.rs\"]",
            "#[path = \"maps/foundation_routes.rs\"]",
            "#[path = \"maps/parent_routes.rs\"]",
            "#[path = \"maps/source_reads.rs\"]",
            "#[path = \"maps/status_metadata.rs\"]",
            "#[path = \"maps/support_routes.rs\"]",
            "#[path = \"maps/typed_error_routes.rs\"]",
            "pub(in super::super) use foundation_routes::*;",
            "pub(in super::super) use parent_routes::*;",
            "pub(in super::super) use source_reads::*;",
            "pub(in super::super) use support_routes::*;",
            "pub(in super::super) use typed_error_routes::*;",
        ],
    );
    for moved_anchor in [
        "pub(in super::super) const STATUS_PARENT",
        "pub(in super::super) const STATUS_REVIEW_FOUNDATION_CHILD",
        "pub(in super::super) const STATUS_REVIEW_TYPED_ERROR_CHILD",
        "pub(in super::super) const STATUS_REVIEW_PLUGIN_IMPORTER_CHILD",
        "pub(in super::super) fn read_status_review_foundation_sources",
        "fn read_review_foundation_sources",
        "fn read_review_typed_error_sources",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "sources/status_maps.rs should delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-map parent routes own root status/date route paths",
        &parent_routes,
        &[
            "STATUS_PARENT",
            "DATE_PARENT",
            "STATUS_REVIEW_CHILD",
            "DATE_REVIEW_CHILD",
        ],
    );
    assert_contains_all(
        "status-map foundation routes own foundation status/date route paths",
        &foundation_routes,
        &[
            "STATUS_REVIEW_FOUNDATION_CHILD",
            "STATUS_REVIEW_FOUNDATION_CHILDREN",
            "DATE_REVIEW_FOUNDATION_CHILD",
            "DATE_REVIEW_FOUNDATION_CHILDREN",
        ],
    );
    assert_contains_all(
        "status-map typed-error routes own typed-error status/date route paths",
        &typed_error_routes,
        &[
            "STATUS_REVIEW_TYPED_ERROR_CHILD",
            "STATUS_REVIEW_TYPED_ERROR_CHILDREN",
            "DATE_REVIEW_TYPED_ERROR_CHILD",
            "DATE_REVIEW_TYPED_ERROR_CHILDREN",
        ],
    );
    assert_contains_all(
        "status-map support routes own non-foundation status/date route paths",
        &support_routes,
        &["STATUS_SUPPORT_CHILD", "DATE_SUPPORT_PLAN_DOC_CHILD"],
    );
    assert_contains_all(
        "status-map source reads own aggregation helpers",
        &source_reads,
        &[
            "read_status_review_foundation_sources",
            "read_date_review_foundation_sources",
            "read_status_review_typed_error_sources",
            "read_date_review_typed_error_sources",
            "child_sources::extend_foundation_expected_slice_sources",
            "child_sources::extend_typed_error_status_doc_sources",
        ],
    );
    assert_contains_all(
        "status-map child source extension owns nested row stems",
        &child_sources,
        &["FOUNDATION_EXPECTED_SLICE_CHILD_STEMS", "status_map_rows"],
    );
    assert_contains_all(
        "status-map status metadata owns status anchors",
        &status_metadata,
        &[
            "SOURCE_STATUS_MAPS_SLICE",
            "SOURCE_STATUS_MAPS_STATUS",
            "SOURCE_STATUS_MAPS_FRAMEWORKS_STATUS",
            "SOURCE_STATUS_MAPS_GUARD",
            "SOURCE_STATUS_MAPS_ROUTE_PATH",
            "SOURCE_STATUS_MAPS_CHILDREN",
        ],
    );
}
fn read_status_map_child(path: &str) -> String {
    read_runtime_src(&format!("tests/runtime_absorption/{path}"))
}
