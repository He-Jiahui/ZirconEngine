use super::Slice;

#[path = "typed_error_structure_rows/child_ownership.rs"]
mod child_ownership;
#[path = "typed_error_structure_rows/core_rows.rs"]
mod core_rows;
#[path = "typed_error_structure_rows/folder_backed.rs"]
mod folder_backed;
#[path = "typed_error_structure_rows/map_rows.rs"]
mod map_rows;
#[path = "typed_error_structure_rows/row_data_owner.rs"]
mod row_data_owner;
#[path = "typed_error_structure_rows/status_doc_delegation_rows.rs"]
mod status_doc_delegation_rows;
#[path = "typed_error_structure_rows/status_doc_path_rows.rs"]
mod status_doc_path_rows;
#[path = "typed_error_structure_rows/status_doc_status_maps_rows.rs"]
mod status_doc_status_maps_rows;
#[path = "typed_error_structure_rows/status_doc_status_mirrors_rows.rs"]
mod status_doc_status_mirrors_rows;
#[path = "typed_error_structure_rows/status_docs.rs"]
mod status_docs;
#[path = "typed_error_structure_rows/structure_assertion_rows.rs"]
mod structure_assertion_rows;
#[path = "typed_error_structure_rows/structure_assertions.rs"]
mod structure_assertions;
#[path = "typed_error_structure_rows/top_level.rs"]
mod top_level;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOC_PATHS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_doc_path_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOC_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_doc_delegation_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOC_STATUS_MAPS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_doc_status_maps_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOC_STATUS_MIRRORS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_doc_status_mirrors_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_assertion_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
