pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs listing prose full inventory sync" => Some(
            "runtime_15_priority_plan_docs_listing_prose_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs moved mirror full inventory sync" => Some(
            "runtime_15_priority_plan_docs_moved_mirror_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs root inventory child split" => Some(
            "runtime_15_priority_plan_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
