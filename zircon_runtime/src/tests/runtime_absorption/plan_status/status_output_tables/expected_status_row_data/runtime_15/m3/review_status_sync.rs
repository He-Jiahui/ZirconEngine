pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "review_status_sync/importer_fixture_rows.rs"]
mod importer_fixture_rows;
#[path = "review_status_sync/p0_core_rows.rs"]
mod p0_core_rows;
#[path = "review_status_sync/provider_lookup_rows.rs"]
mod provider_lookup_rows;
#[path = "review_status_sync/row_data_owner.rs"]
mod row_data_owner;
#[path = "review_status_sync/typed_runtime_rows.rs"]
mod typed_runtime_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    importer_fixture_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const P0_CORE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    p0_core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PROVIDER_LOOKUP_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    provider_lookup_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
