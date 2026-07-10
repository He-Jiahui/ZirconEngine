use super::Slice;

#[path = "row_data_owner/owner_row.rs"]
mod owner_row;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    owner_row::EXPECTED_STATUS_OUTPUT_SLICES;
