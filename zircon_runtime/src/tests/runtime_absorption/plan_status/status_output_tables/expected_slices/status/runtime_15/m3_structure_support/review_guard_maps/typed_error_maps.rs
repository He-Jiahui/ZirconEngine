pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review guard typed-error expected-slice map child split" => Some(
            "runtime_15_review_guard_typed_error_expected_slice_map_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error convergence guard child-owner split" => Some(
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split" => Some(
            "runtime_15_native_plugin_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native ABI surfaces typed-error review guard child-owner split" => Some(
            "runtime_15_native_abi_surfaces_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native plugin descriptor ABI typed-error review guard child-owner split" => Some(
            "runtime_15_native_plugin_descriptor_abi_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI input typed-error review guard child-owner split" => Some(
            "runtime_15_ui_input_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review guard status row-data child-owner split" => {
            Some("runtime_15_review_guard_status_row_data_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review guard row-data topic child-owner split" => {
            Some("runtime_15_review_guard_row_data_topic_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review-guard typed-error row-data child split" => Some(
            "runtime_15_review_guard_typed_error_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code-review row-data owner child split" => Some(
            "runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer row-data owner child split" => Some(
            "runtime_15_plugin_importer_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure row-data child split" => Some(
            "runtime_15_typed_error_structure_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code-review structure-guard row-data folder-backed split" => Some(
            "runtime_15_code_review_structure_guard_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code-review structure-guard root-and-children row-data child split" => {
            Some("runtime_15_code_review_structure_guard_root_and_children_row_data_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings status-row source child-tree sync" => Some(
            "runtime_15_code_review_findings_status_row_source_child_tree_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure status-doc guard child-owner split" => {
            Some("runtime_15_typed_error_structure_status_docs_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split" => {
            Some("runtime_15_typed_error_structure_status_docs_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split" => {
            Some("runtime_15_typed_error_status_doc_mirrors_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error status-doc source helper child split" => {
            Some("runtime_15_typed_error_status_doc_source_helper_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory guard child-owner split" => {
            Some("runtime_15_typed_error_source_inventory_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory guard folder-backed split" => {
            Some("runtime_15_typed_error_source_inventory_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory source helper child split" => {
            Some("runtime_15_typed_error_source_inventory_source_helper_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child sources folder-backed split" => {
            Some("runtime_15_typed_error_source_inventory_child_sources_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child sources structure guard child split" => {
            Some("runtime_15_typed_error_source_inventory_child_sources_structure_guard_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child inventory folder-backed split" => {
            Some("runtime_15_typed_error_source_inventory_child_inventory_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child inventory status-current child split" => {
            Some("runtime_15_typed_error_source_inventory_child_inventory_status_current_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory metadata child split" => {
            Some("runtime_15_typed_error_source_inventory_metadata_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory metadata status-current child split" => {
            Some("runtime_15_typed_error_source_inventory_metadata_status_current_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory delegation child split" => {
            Some("runtime_15_typed_error_source_inventory_delegation_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory delegation folder-backed child split" => {
            Some("runtime_15_typed_error_source_inventory_delegation_folder_backed_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split" => {
            Some("runtime_15_typed_error_source_inventory_delegation_folder_backed_ownership_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 native manifest sources typed-error review guard child-owner split" => Some(
            "runtime_15_native_manifest_sources_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 script host typed-error review guard child-owner split" => Some(
            "runtime_15_script_host_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene world typed-error review guard child-owner split" => Some(
            "runtime_15_scene_world_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset loader typed-error review guard child-owner split" => Some(
            "runtime_15_asset_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset records typed-error review guard child-owner split" => Some(
            "runtime_15_asset_records_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split" => Some(
            "runtime_15_shader_prewarm_cli_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host typed-error review guard child-owner split" => Some(
            "runtime_15_native_live_host_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host lifecycle-paths typed-error review guard child-owner split" => Some(
            "runtime_15_native_live_host_lifecycle_paths_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split" => Some(
            "runtime_15_native_live_host_replay_runtime_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
