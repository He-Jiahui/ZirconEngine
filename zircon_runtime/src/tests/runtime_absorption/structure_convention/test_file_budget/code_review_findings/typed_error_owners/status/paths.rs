#[path = "paths/child_inventory.rs"]
mod child_inventory;
#[path = "paths/review_guard_paths.rs"]
mod review_guard_paths;
#[path = "paths/root_paths.rs"]
mod root_paths;
#[path = "paths/status_current.rs"]
mod status_current;
#[path = "paths/status_slices.rs"]
mod status_slices;

pub(super) use child_inventory::*;
pub(super) use review_guard_paths::*;
pub(super) use root_paths::*;
pub(super) use status_slices::*;

#[test]
fn runtime_15_typed_error_status_doc_paths_are_child_backed() {
    status_current::assert_typed_error_status_doc_paths_are_child_backed();
}
