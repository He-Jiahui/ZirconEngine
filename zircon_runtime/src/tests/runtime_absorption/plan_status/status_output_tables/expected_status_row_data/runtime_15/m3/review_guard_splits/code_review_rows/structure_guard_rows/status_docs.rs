use super::Slice;

#[path = "status_docs/core.rs"]
mod core;
#[path = "status_docs/source_anchors.rs"]
mod source_anchors;
#[path = "status_docs/status_anchor_guard.rs"]
mod status_anchor_guard;
#[path = "status_docs/status_anchors.rs"]
mod status_anchors;

#[rustfmt::skip]
pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    ("Runtime 15 M3 code review findings status-doc guard child-owner split", core::STATUS_DOC_GUARD_CHILD_OWNER_SPLIT),
    ("Runtime 15 M3 code review findings status-doc guard folder-backed split", core::STATUS_DOC_GUARD_FOLDER_BACKED_SPLIT),
    ("Runtime 15 M3 code review findings status-doc status-mirror child-owner split", core::STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_SPLIT),
    ("Runtime 15 M3 code review findings status-doc map-source sync", core::STATUS_DOC_MAP_SOURCE_SYNC),
    ("Runtime 15 M3 code review findings status-doc root inventory child split", core::STATUS_DOC_ROOT_INVENTORY_CHILD_SPLIT),
    ("Runtime 15 M3 code review findings status-doc source anchors child-owner split", source_anchors::STATUS_DOC_SOURCE_ANCHORS_CHILD_OWNER_SPLIT),
    ("Runtime 15 M3 code review findings status-doc source anchors folder-backed split", source_anchors::STATUS_DOC_SOURCE_ANCHORS_FOLDER_BACKED_SPLIT),
    ("Runtime 15 M3 code review findings status-doc status anchors child-owner split", status_anchors::STATUS_DOC_STATUS_ANCHORS_CHILD_OWNER_SPLIT),
    ("Runtime 15 M3 code review findings status-doc status anchors folder-backed split", status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT),
    ("Runtime 15 M3 code review findings status-doc child-anchor list child split", status_anchors::STATUS_DOC_CHILD_ANCHOR_LIST_CHILD_SPLIT),
    ("Runtime 15 M3 code review findings status-doc child-anchor route folder-backed split", status_anchors::STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT),
    ("Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split", status_anchor_guard::STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_SPLIT),
    ("Runtime 15 M3 code review findings status-doc status-anchor child-ownership child split", status_anchor_guard::STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_CHILD_SPLIT),
];
