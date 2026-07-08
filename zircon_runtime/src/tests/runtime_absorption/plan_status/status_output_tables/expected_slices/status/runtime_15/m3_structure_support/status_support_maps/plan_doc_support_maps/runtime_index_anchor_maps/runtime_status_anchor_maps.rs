pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 03 module-doc status index anchor sync" => Some(
            "runtime_15_runtime_03_module_doc_status_index_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 scene asset status anchor sync" => Some(
            "runtime_15_runtime_07_scene_asset_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget status anchor sync" => Some(
            "runtime_15_runtime_07_owner_budget_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 02 generated status anchor sync" => Some(
            "runtime_15_runtime_02_generated_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 10 behavior status anchor sync" => Some(
            "runtime_15_runtime_10_behavior_status_anchor_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

// Guard: runtime_15_runtime_10_behavior_status_index_anchors_are_locked.
// Guard: runtime_15_runtime_02_generated_status_index_anchors_are_locked.
// Guard: runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked.
// Guard: runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked.
// Guard: runtime_15_runtime_03_module_doc_status_index_anchors_are_locked.
