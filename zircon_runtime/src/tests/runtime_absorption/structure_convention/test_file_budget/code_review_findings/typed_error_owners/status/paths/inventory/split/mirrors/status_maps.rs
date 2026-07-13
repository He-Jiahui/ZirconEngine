use super::super::super::super::super::super::super::super::*;
use super::super::super::super::review_guard_paths::{
    REVIEW_GUARD_DATE_MAP_PATH, REVIEW_GUARD_STATUS_MAP_PATH,
    REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH, REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH,
};

const TYPED_ERROR_STATUS_DOC_PATHS_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows/paths_inventory_rows.rs";
const TYPED_ERROR_STATUS_DOC_PATHS_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows/paths_inventory_rows.rs";

pub(super) fn assert_status_maps_contain(
    label: &str,
    split_name: &str,
    split_id: &str,
    split_date: &str,
) {
    let status_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH),
        read_runtime_src(TYPED_ERROR_STATUS_DOC_PATHS_STATUS_MAP_PATH)
    );
    let date_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH),
        read_runtime_src(TYPED_ERROR_STATUS_DOC_PATHS_DATE_MAP_PATH)
    );

    assert_contains_all(
        &format!("M3 review status map records {label}"),
        &status_map,
        &[split_name, split_id],
    );
    assert_contains_all(
        &format!("M3 review date map records {label}"),
        &date_map,
        &[split_name, split_date],
    );
}
