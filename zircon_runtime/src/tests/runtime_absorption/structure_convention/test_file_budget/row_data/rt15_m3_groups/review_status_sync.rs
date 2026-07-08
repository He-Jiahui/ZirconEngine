use super::*;

#[path = "review_status_sync/export_chain.rs"]
mod export_chain;
#[path = "review_status_sync/row_groups.rs"]
mod row_groups;
#[path = "review_status_sync/status_mirrors.rs"]
mod status_mirrors;

#[test]
fn runtime_15_review_status_sync_row_data_is_child_backed() {
    row_groups::assert_review_status_sync_parent_delegates_to_children();
    export_chain::assert_review_status_sync_export_chain_is_current();
    status_mirrors::assert_review_status_sync_status_mirrors_are_current();
}
