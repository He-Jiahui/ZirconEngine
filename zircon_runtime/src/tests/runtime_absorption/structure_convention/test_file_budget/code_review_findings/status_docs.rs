use super::super::*;

#[path = "status/delegation.rs"]
mod delegation;
#[path = "status/root_children.rs"]
mod root_children;
#[path = "status/root_inventory.rs"]
mod root_inventory;
#[path = "status/root_paths.rs"]
mod root_paths;
#[path = "status/root_row_sources.rs"]
mod root_row_sources;
#[path = "status/root_statuses.rs"]
mod root_statuses;
#[path = "status/source_anchor_guard.rs"]
mod source_anchor_guard;
#[path = "status/source_anchors.rs"]
mod source_anchors;
#[path = "status/status_anchor_guard.rs"]
mod status_anchor_guard;
#[path = "status/status_anchors.rs"]
mod status_anchors;
#[path = "status/status_mirrors.rs"]
mod status_mirrors;
#[path = "status/sync.rs"]
mod sync;

pub(super) use root_children::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn assert_code_review_findings_status_docs_are_synced() {
    sync::assert_code_review_findings_status_docs_are_synced();
}

pub(super) fn review_guard_status_rows_source() -> String {
    root_row_sources::review_guard_status_rows_source()
}

pub(super) fn review_guard_status_map_source() -> String {
    root_row_sources::review_guard_status_map_source()
}

pub(super) fn review_guard_date_map_source() -> String {
    root_row_sources::review_guard_date_map_source()
}

pub(super) fn status_doc_route_inventory_source() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        read_runtime_src(STATUS_DOC_PARENT_PATH),
        read_runtime_src(STATUS_DOC_ROOT_PATHS_CHILD),
        read_runtime_src(STATUS_DOC_ROOT_STATUSES_CHILD),
        read_runtime_src(STATUS_DOC_ROOT_CHILDREN_CHILD)
    )
}
