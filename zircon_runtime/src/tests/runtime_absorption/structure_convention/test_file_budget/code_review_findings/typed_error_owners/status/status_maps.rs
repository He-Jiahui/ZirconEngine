use super::super::super::super::*;
use super::*;

#[path = "maps/child_inventory.rs"]
mod child_inventory;
#[path = "maps/review_slices.rs"]
mod review_slices;
#[path = "maps/status_current.rs"]
mod status_current;

pub(super) use child_inventory::*;

pub(super) fn assert_typed_error_status_maps_are_synced(sources: &TypedErrorStatusDocSources) {
    review_slices::assert_typed_error_review_status_maps_are_synced(sources);
}

#[test]
fn runtime_15_typed_error_status_doc_status_maps_are_child_backed() {
    status_current::assert_typed_error_status_maps_are_child_backed();
}
