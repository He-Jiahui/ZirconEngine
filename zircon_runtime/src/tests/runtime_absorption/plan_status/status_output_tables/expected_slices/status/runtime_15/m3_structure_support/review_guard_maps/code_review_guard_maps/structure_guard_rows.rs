pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings structure guard child-owner split" => Some(
            "runtime_15_code_review_findings_structure_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard children folder-backed split" => Some(
            "runtime_15_code_review_findings_structure_guard_children_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard children budget-status child split" => {
            Some("runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard children root inventory child split" => {
            Some("runtime_15_code_review_findings_structure_guard_children_root_inventory_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard children source-map sync" => {
            Some("runtime_15_code_review_findings_structure_guard_children_source_map_sync_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 structure guard plugin-importer child split" => Some(
            "runtime_15_structure_guard_plugin_importer_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard folder-backed summary child-owner split" => {
            Some("runtime_15_code_review_findings_structure_guard_folder_backed_summary_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard folder-backed summary guard folder-backed split" => {
            Some("runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard typed-error child-owner split" => {
            Some("runtime_15_code_review_findings_structure_guard_typed_error_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard typed-error folder-backed split" => {
            Some("runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
