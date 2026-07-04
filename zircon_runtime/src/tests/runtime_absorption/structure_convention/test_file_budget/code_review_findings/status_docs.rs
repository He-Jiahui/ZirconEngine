use super::super::*;

#[path = "status_docs/delegation.rs"]
mod delegation;
#[path = "status_docs/root_children.rs"]
mod root_children;
#[path = "status_docs/root_inventory.rs"]
mod root_inventory;
#[path = "status_docs/root_paths.rs"]
mod root_paths;
#[path = "status_docs/root_row_sources.rs"]
mod root_row_sources;
#[path = "status_docs/root_statuses.rs"]
mod root_statuses;
#[path = "status_docs/source_anchor_guard.rs"]
mod source_anchor_guard;
#[path = "status_docs/source_anchors.rs"]
mod source_anchors;
#[path = "status_docs/status_anchor_guard.rs"]
mod status_anchor_guard;
#[path = "status_docs/status_anchors.rs"]
mod status_anchors;
#[path = "status_docs/status_mirrors.rs"]
mod status_mirrors;
#[path = "status_docs/sync.rs"]
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
