pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs guard child-owner split" => Some(
            "runtime_15_priority_plan_docs_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs child prose full inventory sync" => Some(
            "runtime_15_priority_plan_docs_child_prose_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard-test child-owner split" => Some(
            "runtime_15_priority_plan_docs_guard_test_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard-test child prose full inventory sync" => Some(
            "runtime_15_priority_plan_docs_guard_test_child_prose_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs moved guard path mirror" => Some(
            "runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard inventory row-data source sync" => Some(
            "runtime_15_priority_plan_docs_guard_inventory_row_data_source_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
