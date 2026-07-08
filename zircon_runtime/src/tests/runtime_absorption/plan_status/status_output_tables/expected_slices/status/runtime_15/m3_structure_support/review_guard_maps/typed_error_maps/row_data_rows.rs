pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review guard status row-data child-owner split" => {
            Some("runtime_15_review_guard_status_row_data_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review guard row-data topic child-owner split" => {
            Some("runtime_15_review_guard_row_data_topic_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review-guard typed-error row-data child split" => Some(
            "runtime_15_review_guard_typed_error_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support row-data folder-backed split" => Some(
            "runtime_15_review_guard_status_support_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support folder-backed guard folder-backed split" => Some(
            "runtime_15_review_guard_status_support_folder_backed_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support folder-backed split-layout guard folder-backed split" => Some(
            "runtime_15_review_guard_status_support_folder_backed_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support anchor mirror cleanup" => Some(
            "runtime_15_review_guard_status_support_anchor_mirror_cleanup_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support anchor-mirror cleanup guard folder-backed split" => Some(
            "runtime_15_review_guard_status_support_anchor_mirror_cleanup_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support rows guard folder-backed split" => Some(
            "runtime_15_review_guard_status_support_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support rows split-layout guard folder-backed split" => Some(
            "runtime_15_review_guard_status_support_rows_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error rows guard folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error rows split-layout guard folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_rows_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code-review row-data owner child split" => Some(
            "runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard rows row-data owner child split" => Some(
            "runtime_15_review_guard_rows_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard status-support review rows row-data owner child split" => {
            Some("runtime_15_review_guard_status_support_review_rows_row_data_owner_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review-guard status-support review rows guard folder-backed split" => {
            Some("runtime_15_review_guard_status_support_review_rows_guard_folder_backed_static_passed_cargo_deferred")
        }
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
        "Runtime 15 M3 code review findings split row/map source sync" => Some(
            "runtime_15_code_review_findings_split_row_map_source_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
