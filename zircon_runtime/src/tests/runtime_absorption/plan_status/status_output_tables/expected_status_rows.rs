#[path = "expected_status_row_data.rs"]
mod expected_status_row_data;

pub(super) type ExpectedStatusOutputSlice = (&'static str, &'static [&'static str]);

pub(super) fn expected_status_output_slices() -> impl Iterator<Item = ExpectedStatusOutputSlice> {
    expected_status_row_data::EXPECTED_STATUS_OUTPUT_SLICE_GROUPS
        .iter()
        .flat_map(|group| group.iter().copied())
}
