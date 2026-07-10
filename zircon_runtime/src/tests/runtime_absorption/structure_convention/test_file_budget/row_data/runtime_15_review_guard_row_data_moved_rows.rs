use super::*;

#[path = "runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs"]
mod code_review_rows;
#[path = "runtime_15_review_guard_row_data_moved_rows/delegation.rs"]
mod delegation;
#[path = "runtime_15_review_guard_row_data_moved_rows/root_child_rows.rs"]
mod root_child_rows;
#[path = "runtime_15_review_guard_row_data_moved_rows/root_inventory.rs"]
mod root_inventory;
#[path = "runtime_15_review_guard_row_data_moved_rows/root_paths.rs"]
mod root_paths;
#[path = "runtime_15_review_guard_row_data_moved_rows/root_source_blobs.rs"]
mod root_source_blobs;
#[path = "runtime_15_review_guard_row_data_moved_rows/root_statuses.rs"]
mod root_statuses;
#[path = "runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs"]
mod status_mirrors;
#[path = "runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs"]
mod typed_error_rows;

const MOVED_ROWS_STATUS_ANCHOR_BLOB: &str = "Runtime 15 M3 review-guard row-data moved-row guard child-owner split runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred Runtime 15 M3 review-guard moved-row guard folder-backed split runtime_15_review_guard_moved_row_guard_folder_backed_static_passed_cargo_deferred";

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_source_blobs::*;
pub(super) use root_statuses::*;
