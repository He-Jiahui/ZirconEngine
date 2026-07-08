pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 production file budget guard child-owner split" => Some(
            "runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output variable evidence anchors" => {
            Some("runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output evidence anchors guard folder-backed split" => Some(
            "runtime_15_status_output_evidence_anchors_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 evidence anchors status-mirror child split" => Some(
            "runtime_15_evidence_anchors_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 evidence anchors root inventory child split" => Some(
            "runtime_15_evidence_anchors_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 evidence anchors source/status-map sync" => Some(
            "runtime_15_evidence_anchors_source_status_map_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
