type Slice = super::ExpectedStatusOutputSlice;

#[path = "status_support_rows/review_guard_rows.rs"]
mod review_guard_rows;
#[path = "status_support_rows/source_inventory_delegation_rows.rs"]
mod source_inventory_delegation_rows;
#[path = "status_support_rows/source_inventory_foundation_rows.rs"]
mod source_inventory_foundation_rows;
#[path = "status_support_rows/source_inventory_inventory_metadata_rows.rs"]
mod source_inventory_inventory_metadata_rows;
#[path = "status_support_rows/typed_error_status_doc_rows.rs"]
mod typed_error_status_doc_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_STATUS_SUPPORT_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::STATUS_SUPPORT_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_TYPED_ERROR_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::TYPED_ERROR_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_error_status_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SOURCE_INVENTORY_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    source_inventory_foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SOURCE_INVENTORY_INVENTORY_METADATA_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    source_inventory_inventory_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SOURCE_INVENTORY_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    source_inventory_delegation_rows::EXPECTED_STATUS_OUTPUT_SLICES;
