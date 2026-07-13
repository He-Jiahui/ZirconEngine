pub(super) const FOUNDATION_ROW_DATA_SPLIT_NAME: &str =
    "Runtime 15 M3 status output Runtime 15 foundation row data split";
pub(super) const FOUNDATION_ROW_DATA_SPLIT_ID: &str =
    "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred";
pub(super) const FOUNDATION_TOPIC_SPLIT_NAME: &str =
    "Runtime 15 M3 foundation row-data topic child-owner split";
pub(super) const FOUNDATION_TOPIC_SPLIT_ID: &str =
    "runtime_15_foundation_row_data_topic_child_owner_split_static_passed_cargo_deferred";
pub(super) const FOUNDATION_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 foundation row-data guard child-owner split";
pub(super) const FOUNDATION_GUARD_SPLIT_ID: &str =
    "runtime_15_foundation_row_data_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const STATUS_DOC_SPLIT_NAME: &str =
    "Runtime 15 M3 foundation row-data status-doc guard child-owner split";
pub(super) const STATUS_DOC_SPLIT_ID: &str =
    "runtime_15_foundation_row_data_status_docs_child_owner_split_static_passed_cargo_deferred";
pub(super) const ROW_COUNT_SYNC_NAME: &str = "Runtime 15 M3 foundation row-data 73-row docs sync";
pub(super) const ROW_COUNT_SYNC_ID: &str =
    "runtime_15_foundation_row_data_71_row_docs_sync_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 foundation row-data status-doc guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_foundation_row_data_status_docs_folder_backed_static_passed_cargo_deferred";
pub(super) const ROOT_INVENTORY_STATUS_NAME: &str =
    "Runtime 15 M3 foundation row-data status-doc root inventory child split";
pub(super) const ROOT_INVENTORY_STATUS_ID: &str = "runtime_15_foundation_row_data_status_docs_root_inventory_child_split_static_passed_cargo_deferred";
pub(super) const ROOT_INVENTORY_GUARD_NAME: &str =
    "runtime_15_foundation_row_data_status_docs_root_inventory_is_child_owned";

pub(super) const STATUS_DOC_STATUS_ANCHORS: &[&str] = &[
    STATUS_DOC_SPLIT_NAME,
    STATUS_DOC_SPLIT_ID,
    ROW_COUNT_SYNC_NAME,
    ROW_COUNT_SYNC_ID,
    FOLDER_BACKED_STATUS_NAME,
    FOLDER_BACKED_STATUS_ID,
];
