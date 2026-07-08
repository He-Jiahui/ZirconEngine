use super::*;

#[path = "slice_maps_folder/budget_traversal.rs"]
mod budget_traversal;
#[path = "slice_maps_folder/folder_backed.rs"]
mod folder_backed;
#[path = "slice_maps_folder/route_children.rs"]
mod route_children;
#[path = "slice_maps_folder/status_current.rs"]
mod status_current;

#[test]
fn runtime_15_status_support_expected_slice_owner_paths_are_folder_backed() {
    route_children::assert_expected_slice_owner_path_route_exposes_child_groups();
    budget_traversal::assert_expected_slice_owner_path_budget_traversal_is_current();
    status_current::assert_expected_slice_owner_paths_status_is_current();
}
