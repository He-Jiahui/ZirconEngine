#[path = "review_guard_maps/code_review_guard_maps.rs"]
mod code_review_guard_maps;
#[path = "review_guard_maps/foundation_review_maps.rs"]
mod foundation_review_maps;
#[path = "review_guard_maps/plugin_importer_maps.rs"]
mod plugin_importer_maps;
#[path = "review_guard_maps/top_row_review_maps.rs"]
mod top_row_review_maps;
#[path = "review_guard_maps/typed_error_maps.rs"]
mod typed_error_maps;
#[path = "review_guard_maps/typed_error_structure_maps.rs"]
mod typed_error_structure_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = foundation_review_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = code_review_guard_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = typed_error_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = typed_error_structure_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = plugin_importer_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = top_row_review_maps::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}

// Runtime 15 M3 typed-error structure row-data owner child split anchor mirror:
// runtime_15_typed_error_structure_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs
// runtime_15_typed_error_structure_rows_row_data_owner_is_child_backed
// Runtime 15 M3 code-review row-data owner child split anchor mirror:
// runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs
// runtime_15_code_review_rows_row_data_owner_is_child_backed
// Runtime 15 M3 review-guard rows row-data owner child split anchor mirror:
// runtime_15_review_guard_rows_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/core_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/p0_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/f8_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/late_api_rows.rs
// runtime_15_review_guard_rows_row_data_owner_is_child_backed
// Runtime 15 M3 review-guard status-support review rows row-data owner child split anchor mirror:
// runtime_15_review_guard_status_support_review_rows_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/core_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/status_support_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/typed_error_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/row_data_guard_rows.rs
// runtime_15_review_guard_status_support_review_rows_row_data_owner_is_child_backed
// Runtime 15 M3 review-guard status-support review rows guard folder-backed split anchor mirror:
// runtime_15_review_guard_status_support_review_rows_guard_folder_backed_static_passed_cargo_deferred
// structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows.rs
// structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows/route_children.rs
// structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows/export_chain.rs
// structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows/status_current.rs
// runtime_15_review_guard_status_support_review_rows_guard_is_folder_backed
// Runtime 15 M3 plugin-importer row-data owner child split anchor mirror:
// runtime_15_plugin_importer_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/review_guards.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/status_docs.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/source_inventory.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/row_data_owner.rs
// runtime_15_plugin_importer_rows_row_data_owner_is_child_backed
// Runtime 15 M3 review-guard expected-slice maps folder-backed split anchor mirror:
// runtime_15_review_guard_expected_slice_maps_folder_backed_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/foundation_review_maps.rs
// plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/code_review_guard_maps.rs
// plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps.rs
// plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/plugin_importer_maps.rs
// plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/top_row_review_maps.rs
