pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split" => Some(
            "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure guard folder-backed split" => Some(
            "runtime_15_plugin_importer_dx_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure guard root inventory child split" => Some(
            "runtime_15_plugin_importer_dx_structure_guard_root_inventory_child_split_target_server_direct_binary_passed",
        ),
        "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX status-doc guard folder-backed split" => Some(
            "runtime_15_plugin_importer_dx_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX status-doc root inventory child split" => Some(
            "runtime_15_plugin_importer_dx_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split" => Some(
            "runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split" => {
            Some("runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split" => {
            Some("runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split" => {
            Some("runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split" => {
            Some("runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split" => {
            Some("runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer DX review guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split" => Some(
            "runtime_15_plugin_importer_d13_sdk_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer D1 capability single-source guard folder-backed split" => {
            // runtime_15_plugin_importer_d1_capability_guard_is_folder_backed
            Some("runtime_15_plugin_importer_d1_capability_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer DX source status-map reconciliation" => Some(
            "runtime_15_plugin_importer_dx_source_status_map_reconciliation_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
