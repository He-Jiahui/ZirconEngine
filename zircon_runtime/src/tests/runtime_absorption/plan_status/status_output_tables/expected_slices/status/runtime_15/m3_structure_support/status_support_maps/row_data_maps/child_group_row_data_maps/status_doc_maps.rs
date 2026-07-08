pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-groups status-doc guard child-owner split" => Some(
            "runtime_15_m3_child_groups_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc guard folder-backed split" => Some(
            "runtime_15_m3_child_groups_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc status-mirror child split" => Some(
            "runtime_15_m3_child_groups_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc root inventory child split" => Some(
            "runtime_15_m3_child_groups_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
