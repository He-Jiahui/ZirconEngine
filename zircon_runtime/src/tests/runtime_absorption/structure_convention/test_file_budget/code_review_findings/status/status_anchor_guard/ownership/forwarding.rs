use super::*;

pub(super) fn assert_status_anchor_route_forwards_children(child: &str) {
    assert_contains_all(
        "status-anchor route owns child wiring and helper forwarding",
        child,
        &[
            "#[path = \"status_anchors/child_anchors.rs\"]",
            "mod child_anchors;",
            "#[path = \"status_anchors/map_anchors.rs\"]",
            "mod map_anchors;",
            "pub(super) fn status_doc_child_anchors",
            "pub(super) const STATUS_DOC_MAP_ANCHORS",
            "status_doc_status_anchor_child_source_blob",
            status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
            status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_ID,
        ],
    );
}

pub(super) fn assert_retired_status_anchor_consts_are_absent(child: &str) {
    for retired_const in [
        concat!("pub(super) const STATUS_DOC_", "CHILD_ANCHORS"),
        concat!("pub(super) const STATUS_DOC_", "SESSION_ANCHORS"),
    ] {
        assert!(
            !child.contains(retired_const),
            "status_anchors.rs should expose helper forwarding instead of retired const `{retired_const}`"
        );
    }
}
